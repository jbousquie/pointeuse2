<?php
include_once('crypt.php');

$base_url = "https://filou.iut-rodez.fr/pointeuse/pointe.php?";

$login = $_POST['login'];
$password = $_POST['password'];
$key = $_POST['key'];

$crypto_key = get_crypto_key($login, $key);
$encoded = encode($password, $crypto_key);

$generated_url = $base_url . "l=" . $login . "&k=" . $key . "&e=" . $encoded;
header("Location: ./index.html");
setcookie("url", $generated_url, time() + 86400 * 400);  // expire dans 400 jours
?>