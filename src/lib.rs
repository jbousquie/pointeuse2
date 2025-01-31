use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::blocking;
use scraper::Selector;
use serde_json::Result;

// https://docs.rs/reqwest/0.12.12/reqwest/blocking/struct.Response.html
// https://github.com/rust-scraper/scraper
const URL_OHRIS: &str = "https://ohris.ut-capitole.fr/";
const LEMONLDAP_COOKIE_NAME: &str = "lemonldappdata";
pub fn get_ohris_1() {
    // on fixe la valeur du proxy pour le client
    let proxy = reqwest::Proxy::all("http://cache.iut-rodez.fr:8080/").expect("Proxy invalide");
    // on crée un client avec le proxy et en activant le cookie store
    let client = blocking::Client::builder().proxy(proxy).cookie_store(true).build().expect("Client http invalide");
    // on envoie une requête GET à l'URL de Ohris et on suit les redirections par défaut
    // on doit recevoir le formulaire de login de LemonLDAP comme réponse
    let resp = client.get(URL_OHRIS).send().expect("Erreur de la requête vers Ohris ou de la redirection vers LemonLDAP");
    
    // on récupère le cookie LEMONLDAPAPPDATA qui contient une string de données JSON url-encodées 
    let lemon_ldap_data_cookie = resp.cookies().find(|cookie| cookie.name().eq(LEMONLDAP_COOKIE_NAME)).expect("Aucun cookie LEMONLDAPAPPDATA trouvé dans la dernière redirection");
    let decoded = urlencoding::decode(lemon_ldap_data_cookie.value()).expect("Erreur de décodage url du cookie LEMONLDAPAPPDATA");
    let json_result: Result<serde_json::Value> = serde_json::from_str(&decoded);
    let cookie_object = json_result.expect("Erreur de parsing du JSON du cookie LEMONLDAPAPPDATA");
    // cookie_object contient des paires clé/valeur, certaines sont nécessaires pour remplir les inputs cachés du formulaire de login
    // on récupère les valeurs de _url (encodée en base64) et issuerRequestcasPath
    let cookie_url64 = cookie_object.get("_url").expect("Clé _url non trouvée dans le JSON du cookie LEMONLDAPAPPDATA").as_str().expect("Valeur de la clé _url non convertible en str");
    let url_cas_u8 = BASE64_STANDARD.decode(cookie_url64).expect("Erreur de décodage base64 de l'URL CAS");
    let url_cas = String::from_utf8(url_cas_u8).expect("Erreur de conversion de l'URL CAS en String");

    let issuer_request_caspath = cookie_object.get("issuerRequestcasPath").expect("Clé issuerRequestcasPath non trouvée dans le JSON du cookie LEMONLDAPAPPDATA");
    let login_path = issuer_request_caspath[0].as_str().expect("Valeur de la clé issuerRequestcasPath[0] non convertible en str");
    let login_url = format!("{}/{}?service={}", url_cas, login_path, urlencoding::encode(URL_OHRIS));
    print!("Login URL: {:?}", login_url);

    // on analyse le corps de la réponse du formulaire de login HTML
    let body = resp.text().unwrap();
    let document = scraper::Html::parse_document(&body);
    // On sélectionne tous les tags inputs, cachés ou non, du premier formulaire trouvé
    let form_selector = Selector::parse("form").unwrap();
    let form_element = document.select(&form_selector).next().unwrap();
    let input_selector = Selector::parse("input").unwrap();
    let inputs = form_element.select(&input_selector).map(|input| {
        let name = input.value().attr("name").unwrap();
        let value = input.value().attr("value").unwrap_or("");
        (name, value)
    }).collect::<Vec<_>>();
    println!("{:?}", inputs);

    //client.post()
}
