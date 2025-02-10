use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::blocking;
use scraper::Selector;
use serde_json::Result;

// Reqwest est une bibliothèque HTTP pour Rust
// https://docs.rs/reqwest/0.12.12/reqwest/struct.Client.html
// https://docs.rs/reqwest/0.12.12/reqwest/struct.Request.html
// https://docs.rs/reqwest/0.12.12/reqwest/blocking/struct.Response.html

// Scraper est une bibliothèque de scraping HTML pour Rust
// https://github.com/rust-scraper/scraper

const URL_OHRIS: &str = "https://ohris.ut-capitole.fr/";
const URL_AUTHENTICATED: &str = "https://ohris.ut-capitole.fr/fr/";
const URL_PUNCH: &str = "https://ohris.ut-capitole.fr/fr/time/punch/add_virtual";
const LEMONLDAP_COOKIE_NAME: &str = "lemonldappdata";
const HTML_TOKEN_NAME: &str = "token";

// Le type de message json retourné par la requête xhr GET punch
#[derive(serde::Deserialize)]
struct JsonMsg {
    pub result: String,
    pub message: String,
}

pub fn punch_orhis(login: &str, password: &str) {
    // on fixe la valeur du proxy pour le client
    let proxy = reqwest::Proxy::all("http://cache.iut-rodez.fr:8080/").expect("Proxy invalide");
    // on crée un client avec le proxy et en activant le cookie store
    let client = blocking::Client::builder().proxy(proxy).cookie_store(true).build().expect("Client http invalide");
    // on envoie une requête GET à l'URL de Ohris et on suit les redirections par défaut
    // on doit recevoir le formulaire de login de LemonLDAP comme réponse
    let resp = client
                            .get(URL_OHRIS)
                            .header("Accept","text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
                            .header("Accept-Language", "fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7")
                            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15")
                            .header("Cache-Control", "max-age=0")
                            .header("Connection", "keep-alive")
                            .send()
                            .expect("Erreur de la requête vers Ohris ou de la redirection vers LemonLDAP");
    
    // on récupère le cookie LEMONLDAPAPPDATA qui contient une string de données JSON url-encodées 
    let lemon_ldap_data_cookie = resp.cookies().find(|cookie| cookie.name().eq(LEMONLDAP_COOKIE_NAME)).expect("Aucun cookie LEMONLDAPAPPDATA trouvé dans la dernière redirection");
    let decoded = urlencoding::decode(lemon_ldap_data_cookie.value()).expect("Erreur de décodage url du cookie LEMONLDAPAPPDATA");
    let json_result: Result<serde_json::Value> = serde_json::from_str(&decoded);
    let cookie_object = json_result.expect("Erreur de parsing du JSON du cookie LEMONLDAPAPPDATA");
    // cookie_object contient des paires clé/valeur, certaines sont nécessaires pour remplir les inputs cachés du formulaire de login
    // on récupère les valeurs de _url (encodée en base64) et issuerRequestcasPath qui vont nous servir à construire l'URL du action POST
    let cookie_url64 = cookie_object.get("_url").expect("Clé _url non trouvée dans le JSON du cookie LEMONLDAPAPPDATA").as_str().expect("Valeur de la clé _url non convertible en str");
    let url_cas_u8 = BASE64_STANDARD.decode(cookie_url64).expect("Erreur de décodage base64 de l'URL CAS");
    let url_cas = String::from_utf8(url_cas_u8).expect("Erreur de conversion de l'URL CAS en String");

    let issuer_request_caspath = cookie_object.get("issuerRequestcasPath").expect("Clé issuerRequestcasPath non trouvée dans le JSON du cookie LEMONLDAPAPPDATA");
    let login_path = issuer_request_caspath[0].as_str().expect("Valeur de la clé issuerRequestcasPath[0] non convertible en str");
    let login_url = format!("{}/{}?service={}", url_cas, login_path, urlencoding::encode(URL_OHRIS));


    // Scraping du formulaire de login LemonLDAP
    // on analyse le corps de la réponse du formulaire de login HTML
    let body = resp.text().unwrap();
    let document = scraper::Html::parse_document(&body);
    // On sélectionne tous les tags inputs, cachés ou non, du premier formulaire trouvé
    let form_selector = Selector::parse("form").unwrap();
    let form_element = document.select(&form_selector).next().unwrap();
    let input_selector = Selector::parse("input").unwrap();
    // On récupère le token de sécurité dans l'un des inputs cachés du formulaire
    let is_input_token = form_element.select(&input_selector).find(|input| {
        let name = input.value().attr("name").expect("Attribut name non trouvé dans un des champs input");
        name.eq(HTML_TOKEN_NAME)
    });
    let token = if let Some(input_token) = is_input_token {
        input_token.value().attr("value").expect("Attribut value non trouvé dans le champ input token")
    } else {
        println!("Token non trouvé dans le formulaire de login");
        return;
    };

    // Evvoi de la requête d'authentification POST sur LemonLDAP
    let auth_resp = client
        .post(login_url).
        form(&[
        ("user", login),
        ("password", password),
        ("timezone", "1"),
        ("skin", "ut-capitole.fr"),
        ("token", token),
        ])
        .header("Accept","text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15")
        .header("Cache-Control", "max-age=0")
        .header("Connection", "keep-alive")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .expect("Erreur de la requête POST d'authentification sur LemonLDAP");

    // On s'assure que l'authentification a réussi en vérifiant la redirection vers la page d'accueil de Ohris
    let final_url = auth_resp.url().as_str(); 
    let final_status = auth_resp.status();
    if final_url.ne(URL_AUTHENTICATED) || !final_status.is_success() {
        println!("Authentification CAS échouée");
        return;
    }

    // Envoie de la requête xhr GET et récupération du résultat
    let xhr_resp = client
                                .get(URL_PUNCH)
                                .header("X-Requested-With", "XMLHttpRequest")
                                .header("Accept","application/json, text/javascript, */*; q=0.01")
                                .header("Accept-Language", "fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7")
                                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15")
                                .header("Cache-Control", "max-age=0")
                                .header("Connection", "keep-alive")
                                .send()
                                .expect("Erreur de la requête GET xhr de punch");
    let json: JsonMsg = xhr_resp.json().expect("Erreur de récupération du JSON de la réponse xhr");
    println!("{}\n{}", json.message, json.result);

}
