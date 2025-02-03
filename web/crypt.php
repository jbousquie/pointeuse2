<?php
function encode($password, $key) {
    $iv = get_iv();
    $enc = openssl_encrypt($password, 'DES-EDE3-CBC', $key, 0, $iv);
    return $enc;
};

function decode() {
    $iv = get_iv();
    $dec = openssl_decrypt($encoded, 'DES-EDE3-CBC', $key, 0, $iv);
    return $dec;
};

function get_crypto_key($login, $key) {
    $conc = $key . $login;
    return substr($conc, 0, 8);
};

function get_iv() {
    $val = M_EULER + M_PI;
    $dec = str($val - floor($val));
    return substr($val, 2, 10);
};
?>