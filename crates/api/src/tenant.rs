use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TenantCtx {
    pub tenant_id: Uuid,
    pub role: String,
    pub key_id: Option<Uuid>,
    pub subject: String,
}

impl TenantCtx {
    pub fn local_default(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            role: "platform_admin".into(),
            key_id: None,
            subject: "local-operator".into(),
        }
    }
}
