use serde::{Deserialize, Serialize};

/// Authorization Serverが認証済みとして受け取る主体。
///
/// 本人確認と値の検査は利用側が行う。中核はissuerとsubjectをtokenへ結び付けるためだけに使う。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Principal {
    issuer: String,
    subject: String,
}

impl Principal {
    /// 検証済みのissuerとsubjectから主体を作る。
    pub fn new(issuer: String, subject: String) -> Self {
        Self { issuer, subject }
    }

    /// 主体を認証したissuerを返す。
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    /// issuer内で一意なsubjectを返す。
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

/// OAuth clientの検証対象となる登録情報。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Client {
    /// 登録済みclient IDまたはClient ID Metadata Document URL。
    pub client_id: String,
    /// 同意画面へ表示するclient名。
    pub display_name: String,
    /// 登録したredirect URIの一覧。
    pub redirect_uris: Vec<String>,
}

/// DCRでclientが申告するOpenID Connect application type。
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApplicationType {
    /// 利用者の端末で動作する公開client。
    Native,
    /// Web server上で動作する公開client。
    Web,
}

/// RFC 7591互換endpointで受け取る公開clientのmetadata。
#[derive(Deserialize)]
pub struct DynamicClientRegistrationRequest {
    /// clientが申告した表示名。
    pub client_name: Option<String>,
    /// clientが申告したredirect URI。
    pub redirect_uris: Option<Vec<String>>,
    /// token endpointで使う認証方式。
    pub token_endpoint_auth_method: Option<String>,
    /// clientが使用するgrant type。
    pub grant_types: Option<Vec<String>>,
    /// clientが使用するresponse type。
    pub response_types: Option<Vec<String>>,
    /// clientの実行形態。
    pub application_type: Option<ApplicationType>,
}

/// DCRで登録した公開clientのmetadata。
#[derive(Serialize)]
pub struct DynamicClientRegistrationResponse {
    /// Authorization Serverが発行したclient ID。
    pub client_id: String,
    /// 検証後の表示名。
    pub client_name: String,
    /// 登録したredirect URI。
    pub redirect_uris: Vec<String>,
    /// 公開clientに固定する認証方式。
    pub token_endpoint_auth_method: &'static str,
    /// 対応するgrant type。
    pub grant_types: [&'static str; 2],
    /// 対応するresponse type。
    pub response_types: [&'static str; 1],
    /// clientの実行形態。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_type: Option<ApplicationType>,
}

impl DynamicClientRegistrationResponse {
    /// 検証後のclient情報から登録応答を作る。
    pub fn new(client: Client, application_type: Option<ApplicationType>) -> Self {
        Self {
            client_id: client.client_id,
            client_name: client.display_name,
            redirect_uris: client.redirect_uris,
            token_endpoint_auth_method: "none",
            grant_types: ["authorization_code", "refresh_token"],
            response_types: ["code"],
            application_type,
        }
    }
}

/// 構文と対応範囲を検証した動的client登録。
pub struct ValidatedDynamicClientRegistration {
    /// 保存できるclient情報。
    pub client: Client,
    /// clientの実行形態。
    pub application_type: Option<ApplicationType>,
}

/// 動的client登録を拒否した理由。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynamicClientRegistrationError {
    /// redirect URIが一つも指定されていない状態。
    MissingRedirectUris,
    /// Authorization Serverが対応しないmetadataの指定。
    UnsupportedMetadata,
}

/// 登録情報との照合を終えたredirect URIと、認可要求での指定有無。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolvedRedirectUri {
    /// 認可要求が明示し、登録情報と一致したURI。
    Supplied(String),
    /// 登録済みURIが一つだけだったため補ったURI。
    Inferred(String),
}

impl ResolvedRedirectUri {
    /// 解決後のredirect URIを返す。
    pub fn as_str(&self) -> &str {
        match self {
            Self::Supplied(value) | Self::Inferred(value) => value,
        }
    }

    /// 認可要求で明示されていたかを返す。
    pub fn was_supplied(&self) -> bool {
        matches!(self, Self::Supplied(_))
    }
}

/// 認可codeとtokenへ保存する、利用者、client、resource、scopeの結合。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationGrant {
    /// grantを承認した主体。
    pub principal: Principal,
    /// grantを受け取るclient ID。
    pub client_id: String,
    /// 認可応答のredirect URI。
    pub redirect_uri: ResolvedRedirectUri,
    /// tokenを使用できるresource URI。
    pub resource_uri: String,
    /// grantへ付与したscope。
    pub scopes: Vec<String>,
}

/// 有効なaccess tokenから復元した主体とscope。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedPrincipal {
    /// access tokenに結び付いた主体。
    pub principal: Principal,
    /// access tokenへ付与されたscope。
    pub scopes: Vec<String>,
}

/// Unix epochからのミリ秒。保存adapterとの時刻の受け渡しに使う。
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Unix epochからのミリ秒を包む。
    pub const fn new(milliseconds: i64) -> Self {
        Self(milliseconds)
    }

    /// Unix epochからのミリ秒を返す。
    pub const fn get(self) -> i64 {
        self.0
    }
}

/// Authorization Serverの状態遷移が返す失敗分類。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationError {
    /// 要求の形式またはPKCE値が不正な状態。
    InvalidRequest,
    /// clientを解決または検証できない状態。
    InvalidClient,
    /// redirect URIが登録情報と一致しない状態。
    InvalidRedirectUri,
    /// scopeが未対応または上限外である状態。
    InvalidScope,
    /// resource URIが対象resourceと一致しない状態。
    InvalidTarget,
    /// code、refresh tokenまたはbindingが無効な状態。
    InvalidGrant,
    /// 動的client登録の保存上限へ達した状態。
    Capacity,
    /// repositoryまたはclient metadata取得を利用できない状態。
    Unavailable,
}

/// Authorization Code Flowでtransportから受け取る未検証の認可要求。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationRequest {
    /// 要求元のclient ID。
    pub client_id: String,
    /// 要求元が明示したredirect URI。
    pub redirect_uri: Option<String>,
    /// tokenの使用先resource URI。
    pub resource_uri: String,
    /// 要求したscope。空の場合はresourceの既定scopeを解決する。
    pub scopes: Vec<String>,
    /// PKCE S256 code challenge。
    pub code_challenge: String,
}

/// 保存済み登録またはCIMDから解決したclientとredirect URI。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationClient {
    /// 検証対象のclient情報。
    pub client: Client,
    /// client情報を得た方式。
    pub registration_method: ClientRegistrationMethod,
    /// 認可要求に使用するredirect URI。
    pub redirect_uri: String,
}

/// Authorization Serverがclient情報を得た方式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientRegistrationMethod {
    /// 動的client登録endpointによる登録。
    Dynamic,
    /// Client ID Metadata Documentによる解決。
    MetadataDocument,
}

/// 永続化したclientと、その情報を更新する際に使う登録方式。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredClient {
    /// 永続化するclient情報。
    pub client: Client,
    /// client情報を更新・再解決するときに使う登録方式。
    pub registration_method: ClientRegistrationMethod,
}

/// client登録とredirect URIを照合し、既定scopeも解決した認可要求。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedAuthorizationRequest {
    /// 検証済みのclient情報。
    pub client: Client,
    /// client情報を得た方式。
    pub registration_method: ClientRegistrationMethod,
    /// 登録情報と照合したredirect URI。
    pub redirect_uri: ResolvedRedirectUri,
    /// policyと一致したresource URI。
    pub resource_uri: String,
    /// 対応範囲内へ正規化した要求scope。
    pub scopes: Vec<String>,
    /// 検証済みのPKCE S256 code challenge。
    pub code_challenge: String,
}

/// token endpointだけが短時間保持するtoken pair。秘密値のためDebugを実装しない。
pub struct TokenPair {
    /// clientへ一度だけ返すaccess token平文。
    pub access_token: String,
    /// clientへ一度だけ返すrefresh token平文。
    pub refresh_token: String,
    /// access tokenの残り有効秒数。
    pub access_expires_in_seconds: u64,
    /// 空白区切りで正規化したscope。
    pub scope: String,
}

/// OAuth token endpointが公開するBearer token応答。
#[derive(Serialize)]
pub struct TokenResponse {
    /// Bearer access token。
    pub access_token: String,
    /// refresh token。
    pub refresh_token: String,
    /// `Bearer`に固定したtoken type。
    pub token_type: &'static str,
    /// access tokenの残り有効秒数。
    pub expires_in: u64,
    /// 空白区切りで正規化したscope。
    pub scope: String,
}

impl From<TokenPair> for TokenResponse {
    fn from(pair: TokenPair) -> Self {
        Self {
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
            token_type: "Bearer",
            expires_in: pair.access_expires_in_seconds,
            scope: pair.scope,
        }
    }
}

/// refresh token rotationでrepositoryへ渡す、生成済みの新旧tokenとbinding。
pub struct RefreshTokenRotation {
    /// 一回消費する現在のrefresh token。
    pub refresh_token: String,
    /// tokenを発行したclient ID。
    pub client_id: String,
    /// tokenの使用先resource URI。
    pub resource_uri: String,
    /// 更新時に縮小を要求したscope。
    pub requested_scopes: Option<Vec<String>>,
    /// rotation成功時に保存する新しいaccess token。
    pub new_access_token: String,
    /// rotation成功時に保存する新しいrefresh token。
    pub new_refresh_token: String,
    /// 新しいaccess tokenの失効時刻。
    pub access_expires_at: Timestamp,
    /// 新しいrefresh tokenの失効時刻。
    pub refresh_expires_at: Timestamp,
}

/// 認可codeの一回消費とtoken pair発行を同じtransactionで行うための入力。
pub struct AuthorizationCodeExchange {
    /// 一回消費する認可code。
    pub code: String,
    /// codeに結び付くclient ID。
    pub client_id: String,
    /// codeに結び付くredirect URI。
    pub redirect_uri: Option<String>,
    /// tokenの使用先resource URI。
    pub resource_uri: String,
    /// verifierから計算したPKCE S256 challenge。
    pub code_challenge: String,
    /// 交換成功時に保存するaccess token。
    pub access_token: String,
    /// 交換成功時に保存するrefresh token。
    pub refresh_token: String,
    /// access tokenの失効時刻。
    pub access_expires_at: Timestamp,
    /// refresh tokenの失効時刻。
    pub refresh_expires_at: Timestamp,
}

/// repositoryが返すrefresh token rotationの結果。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefreshTokenRotationOutcome {
    /// 新しいtoken pairを保存した状態。
    Rotated {
        /// 新しいaccess tokenへ付与したscope。
        access_scopes: Vec<String>,
    },
    /// tokenが不明、期限切れ、使用済み、またはbinding不一致である状態。
    InvalidToken,
    /// 要求scopeが元のgrantを拡大する状態。
    InvalidScope,
}
