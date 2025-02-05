<?php
function encode($password, $key) {
    $iv = get_iv();
    $enc = urlencode(openssl_encrypt($password, 'DES-EDE3-CBC', $key, 0, $iv));
    return $enc;
};

function decode($encoded, $key) {
    $iv = get_iv();
    $dec = urldecode(openssl_decrypt($encoded, 'DES-EDE3-CBC', $key, 0, $iv));
    return $dec;
};

function get_crypto_key($login, $key) {
    $conc = $key . $login;
    return substr($conc, 0, 8);
};

function get_iv() {
    $val = M_EULER * M_PI;
    $dec = strval($val - floor($val));
    return substr($dec, 2, 11);
};
?>