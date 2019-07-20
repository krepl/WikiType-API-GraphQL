pub fn get_google_oauth2_public_key(key_id: &str) -> biscuit::jws::Secret {
    // TODO: replace by querying https://www.googleapis.com/oauth2/v1/certs
    // Can cache the latest certificate.
    //
    // examine the Cache-Control header in the response to determine when you
    // should retrieve them again.
    //
    // See: https://developers.google.com/identity/sign-in/web/backend-auth#verify-the-integrity-of-the-id-token

    let future = futures::future::poll_fn(|| {
        tokio_threadpool::blocking(|| {
            let http_client = reqwest::Client::new();
            let issuer = oidc::issuer::google();
            let config = oidc::discovery::discover(&http_client, issuer).unwrap();
            let jwks = oidc::discovery::jwks(&http_client, config.jwks_uri.clone()).unwrap();
            jwks
        })
    });

    use futures::future::Future;

    let jwks = future.wait().unwrap();

    let public_key = jwks.find(key_id).unwrap();

    match &public_key.algorithm {
        biscuit::jwk::AlgorithmParameters::RSA(key_params) => {
            biscuit::jws::Secret::RSAModulusExponent {
                n: key_params.n.clone(),
                e: key_params.e.clone(),
            }
        }
        _ => panic!(),
    }
}

/// The primary extension that OpenID Connect makes to OAuth 2.0 to enable End-Users to be
/// Authenticated.
///
/// The ID Token is a security token that contains Claims about the Authentication of an End-User
/// by an Authorization Server when using a Client, and potentially other requested Claims. The ID
/// Token is represented as a JSON Web Token (JWT).
///
/// See [ID Token](https://openid.net/specs/openid-connect-core-1_0.html#IDToken) and
/// [Standard Claims](https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims).
#[derive(Debug, Serialize, Deserialize)]
pub struct IdToken {
    /// Issuer Identifier.
    pub iss: String,

    /// Subject Identifier.
    pub sub: String,

    /// Audience(s) for which this ID Token is intended.
    pub aud: String,

    /// Expiration time on or after which the ID Token MUST NOT be accepted for processing.
    pub exp: i64,

    /// Time at which the JWT was issued.
    pub iat: i64,

    /// Time when the End-User authentication occurred.
    pub auth_time: Option<String>,

    /// String value used to associate a Client session with an ID Token, and to mitigate replay
    /// attacks.
    pub nonce: Option<String>,

    /// Authentication Context Class Reference.
    pub acr: Option<String>,

    /// Authentication Methods References.
    pub amr: Option<String>,

    /// Authorized party.
    pub azp: Option<String>,

    /// Access Token hash value.
    pub at_hash: Option<String>,

    /// A unique identifier for the token, which can be used to prevent reuse of the token.
    pub jti: Option<String>,

    /// End-User's full name in displayable form.
    pub name: Option<String>,

    /// URL of the End-User's profile picture.
    pub picture: Option<String>,

    /// Given name(s) or first name(s) of the End-User.
    pub given_name: Option<String>,

    /// Surname(s) or last name(s) of the End-User.
    pub family_name: Option<String>,

    /// End-User's locale, represented as a BCP47 [RFC5646](https://tools.ietf.org/html/rfc5646)
    /// language tag.
    pub locale: Option<String>,
}
