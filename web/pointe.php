<?php
include_once('crypt.php');

$login = $_GET['l'];
$key = $_GET['k'];
$encoded =$_GET['e'];

$crypto_key = get_crypto_key($login, $key);
$password = decode($encoded, $crypto_key);

$exe_path = "/home/jerome/scripts/rust/pointeuse2/target/release/pointeuse2";
$command = $exe_path . " " . $login . " " . $password;
$output = array();
$return_var = 0;
exec($command, $output, $return_var);
$len = count($output);
if ($len == 0) {
    echo("Pas de réponse de Filou");
}
else {
    echo($output[0]);
}
?>