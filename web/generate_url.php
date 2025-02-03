<?php
include_once('crypt.php');

$login = $_POST['login'];
$password = $_POST['password'];
$key = $_POST['key'];

$crypto_key = get_crypto_key($login, $key);
$encoded = encode($password, $crypto_key);

echo($login . $encoded);
?>