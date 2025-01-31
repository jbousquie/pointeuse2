use reqwest::blocking;
use scraper::Selector;
use serde_json::Result;

// https://docs.rs/reqwest/0.12.12/reqwest/blocking/struct.Response.html
// https://github.com/rust-scraper/scraper
const LEMONLDAP_COOKIE_NAME: &str = "lemonldappdata";
pub fn get_ohris_1() {
    // on fixe la valeur du proxy pour le client
    let proxy = reqwest::Proxy::all("http://cache.iut-rodez.fr:8080/").expect("Proxy invalide");
    // on crée un client avec le proxy et en activant le cookie store
    let client = blocking::Client::builder().proxy(proxy).cookie_store(true).build().expect("Client http invalide");
    // on envoie une requête GET à l'URL de Ohris et on suit les redirections par défaut
    // on doit recevoir le formulaire de login de LemonLDAP comme réponse
    let resp = client.get("https://ohris.ut-capitole.fr/").send().expect("Erreur de la requête vers Ohris ou de la redirection vers LemonLDAP");
    
    // on récupère le cookie LEMONLDAPAPPDATA qui contient une string de données JSON url-encodées 
    let lemon_ldap_data_cookie = resp.cookies().find(|cookie| cookie.name().eq(LEMONLDAP_COOKIE_NAME)).expect("Aucun cookie LEMONLDAPAPPDATA trouvé dans la dernière redirection");
    let decoded = urlencoding::decode(lemon_ldap_data_cookie.value()).expect("Erreur de décodage url du cookie LEMONLDAPAPPDATA");
    let json_result: Result<serde_json::Value> = serde_json::from_str(&decoded);
    let cookie_object = json_result.expect("Erreur de parsing du JSON du cookie LEMONLDAPAPPDATA");
    // cookie_object contient des paires clé/valeur, certaines sont nécessaires pour remplir les inputs cachés du formulaire de login
    
    
    // on analyse le corps de la réponse HTML
    let body = resp.text().unwrap();
    let document = scraper::Html::parse_document(&body);
    // On sélectionne tous les tags inputs, cachés ou non, du premier formulaire trouvé
    let form_selector = Selector::parse("form").unwrap();
    let form_element = document.select(&form_selector).next().unwrap();
    let input_selector = Selector::parse("input").unwrap();
    let inputs = form_element.select(&input_selector).map(|input| {
        let name = input.value().attr("name").unwrap();
        //let value = input.value().attr("value").unwrap();
        name
    }).collect::<Vec<_>>();
    println!("{:?}", inputs);
}
