use:
openssl req -x509 -newkey rsa:4096 -keyout ssl/key.pem -out ssl/cert.pem -days 365 -nodes -subj "/CN=localhost" -addext "basicConstraints=critical,CA:FALSE"  -addext "subjectAltName=IP:127.0.0.1,DNS:localhost" -addext "keyUsage=critical,digitalSignature,keyEncipherment" -addext "extendedKeyUsage=serverAuth"
to generate ssl keys and certificates for localy hosted connections

if your using windows you may need to set the openssl config file location
this generally looks like this
set OPENSSL_CONF=C:\Program Files\OpenSSL-Win64\bin\openssl.cfg

though check for your config files location before hand
