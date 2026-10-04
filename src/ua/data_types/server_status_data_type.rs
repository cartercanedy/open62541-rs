use crate::ua;

crate::data_type::ua_struct! {
    pub struct ServerStatusDataType {
        buildInfo: &ua::BuildInfo,
        state: &ua::ServerState,
        #[from_inner]
        startTime: ua::DateTime,
        #[from_inner]
        currentTime: ua::DateTime,
        secondsTillShutdown: u32,
        shutdownReason: &ua::LocalizedText
    }
}
