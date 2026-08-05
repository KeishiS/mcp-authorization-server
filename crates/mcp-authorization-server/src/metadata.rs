use serde::Serialize;

use crate::ResourcePolicy;

/// Authorization Server Metadataに公開するendpoint。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationServerEndpoints {
    /// Authorization Serverのissuer URL。
    pub issuer: String,
    /// 利用者の認可を開始するendpoint。
    pub authorization_endpoint: String,
    /// code交換とrefreshを受け付けるendpoint。
    pub token_endpoint: String,
    /// token取消を受け付けるendpoint。
    pub revocation_endpoint: String,
    /// 動的client登録を受け付けるendpoint。
    pub registration_endpoint: String,
}

/// RFC 9728のProtected Resource Metadataで公開する対応範囲。
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProtectedResourceMetadata {
    /// 保護対象のresource URI。
    pub resource: String,
    /// 利用者へ表示するresource名。
    pub resource_name: String,
    /// このresourceを担当するAuthorization Server。
    pub authorization_servers: Vec<String>,
    /// access tokenをHTTP headerで送ることを示す値。
    pub bearer_methods_supported: [&'static str; 1],
    /// resourceが理解するscope。
    pub scopes_supported: Vec<String>,
}

/// RFC 8414とMCPの認可要件に従って公開するAuthorization Server Metadata。
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AuthorizationServerMetadata {
    /// Authorization Serverのissuer URL。
    pub issuer: String,
    /// 利用者の認可を開始するendpoint。
    pub authorization_endpoint: String,
    /// code交換とrefreshを受け付けるendpoint。
    pub token_endpoint: String,
    /// token取消を受け付けるendpoint。
    pub revocation_endpoint: String,
    /// 動的client登録を受け付けるendpoint。
    pub registration_endpoint: String,
    /// このAuthorization Serverが担当するresource URI。
    pub protected_resources: Vec<String>,
    /// 発行できるscope。
    pub scopes_supported: Vec<String>,
    /// 対応するOAuth response type。
    pub response_types_supported: [&'static str; 1],
    /// 対応するOAuth grant type。
    pub grant_types_supported: [&'static str; 2],
    /// 対応するPKCE challenge方式。
    pub code_challenge_methods_supported: [&'static str; 1],
    /// token endpointで対応するclient認証方式。
    pub token_endpoint_auth_methods_supported: [&'static str; 1],
    /// revocation endpointで対応するclient認証方式。
    pub revocation_endpoint_auth_methods_supported: [&'static str; 1],
    /// 認可応答へissuer識別子を含めることを示す値。
    pub authorization_response_iss_parameter_supported: bool,
    /// Client ID Metadata Documentを解決できることを示す値。
    pub client_id_metadata_document_supported: bool,
}

impl ResourcePolicy {
    /// このresource policyからRFC 9728 metadataを作る。
    pub fn protected_resource_metadata(
        &self,
        authorization_server: String,
    ) -> ProtectedResourceMetadata {
        ProtectedResourceMetadata {
            resource: self.uri().to_string(),
            resource_name: self.display_name().to_owned(),
            authorization_servers: vec![authorization_server],
            bearer_methods_supported: ["header"],
            scopes_supported: self.supported_scopes().to_vec(),
        }
    }

    /// このresource policyとendpointからRFC 8414 metadataを作る。
    pub fn authorization_server_metadata(
        &self,
        endpoints: &AuthorizationServerEndpoints,
    ) -> AuthorizationServerMetadata {
        AuthorizationServerMetadata {
            issuer: endpoints.issuer.clone(),
            authorization_endpoint: endpoints.authorization_endpoint.clone(),
            token_endpoint: endpoints.token_endpoint.clone(),
            revocation_endpoint: endpoints.revocation_endpoint.clone(),
            registration_endpoint: endpoints.registration_endpoint.clone(),
            protected_resources: vec![self.uri().to_string()],
            scopes_supported: self.supported_scopes().to_vec(),
            response_types_supported: ["code"],
            grant_types_supported: ["authorization_code", "refresh_token"],
            code_challenge_methods_supported: ["S256"],
            token_endpoint_auth_methods_supported: ["none"],
            revocation_endpoint_auth_methods_supported: ["none"],
            authorization_response_iss_parameter_supported: true,
            client_id_metadata_document_supported: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_metadata_documents_use_the_same_resource_policy() {
        let policy = ResourcePolicy::new(
            "https://catalog.example/mcp".to_owned(),
            "Catalog search".to_owned(),
            vec!["catalog:search".to_owned(), "catalog:export".to_owned()],
            vec!["catalog:search".to_owned()],
        )
        .expect("resource policy");
        let endpoints = AuthorizationServerEndpoints {
            issuer: "https://catalog.example/".to_owned(),
            authorization_endpoint: "https://catalog.example/oauth/authorize".to_owned(),
            token_endpoint: "https://catalog.example/oauth/token".to_owned(),
            revocation_endpoint: "https://catalog.example/oauth/revoke".to_owned(),
            registration_endpoint: "https://catalog.example/oauth/register".to_owned(),
        };

        let resource = policy.protected_resource_metadata(endpoints.issuer.clone());
        let server = policy.authorization_server_metadata(&endpoints);
        assert_eq!(resource.resource, "https://catalog.example/mcp");
        assert_eq!(resource.resource_name, "Catalog search");
        assert_eq!(resource.scopes_supported, server.scopes_supported);
        assert_eq!(server.protected_resources, [resource.resource]);
    }
}
