// This file is @generated by prost-build.
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Participant {
    /// Local(Competition) user_id; Данное поле НЕ соответствует каким-либо uid в системе авторизации и аунтефикации.
    #[prost(int32, tag = "1")]
    pub uid: i32,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(enumeration = "participant::Gender", tag = "3")]
    pub gender: i32,
    #[prost(message, optional, tag = "4")]
    pub birthdate: ::core::option::Option<::prost_wkt_types::Timestamp>,
    #[prost(string, repeated, tag = "5")]
    pub extra_personal: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `Participant`.
pub mod participant {
    #[derive(serde::Deserialize, serde::Serialize)]
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Gender {
        /// типа паркетный
        Unknown = 0,
        /// ну понятно
        Male = 1,
        /// очевидно
        Female = 2,
    }
    impl Gender {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Gender::Unknown => "Unknown",
                Gender::Male => "Male",
                Gender::Female => "Female",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "Unknown" => Some(Self::Unknown),
                "Male" => Some(Self::Male),
                "Female" => Some(Self::Female),
                _ => None,
            }
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EasyParticipant {
    /// int32 uid = 1; // Local(Competition) user_id; Данное поле НЕ соответствует каким-либо uid в системе авторизации и аунтефикации.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub extra_personal: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Team {
    /// team id ~ local id;
    #[prost(int32, tag = "1")]
    pub tid: i32,
    /// Nomination
    #[prost(string, tag = "2")]
    pub nom: ::prost::alloc::string::String,
    /// Org presented
    #[prost(string, tag = "3")]
    pub organisation: ::prost::alloc::string::String,
    /// Participants Ids
    #[prost(message, repeated, tag = "4")]
    pub participants: ::prost::alloc::vec::Vec<EasyParticipant>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NominationDeclaration {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// action id <-> team
    #[prost(map = "int32, message", tag = "2")]
    pub teams: ::std::collections::HashMap<i32, Team>,
    /// index = order, items = team ids
    ///
    /// generic.IntPair ages = 2;
    /// int64   group_size   = 3;
    #[prost(int32, repeated, tag = "3")]
    pub inner_queue: ::prost::alloc::vec::Vec<i32>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompetitionQueue {
    #[prost(int32, tag = "1")]
    pub id: i32,
    #[prost(message, repeated, tag = "2")]
    pub nomination_list: ::prost::alloc::vec::Vec<NominationDeclaration>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompDeclaration {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub public: bool,
    #[prost(int32, tag = "3")]
    pub related_organisation_id: i32,
    #[prost(message, optional, tag = "4")]
    pub dates: ::core::option::Option<super::generic::DatePair>,
    #[prost(string, optional, tag = "5")]
    pub place: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub descr: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(enumeration = "JudgeScheme", tag = "7")]
    pub scheme: i32,
    /// repeated Participant part_list = 9;
    #[prost(message, repeated, tag = "10")]
    pub queues: ::prost::alloc::vec::Vec<CompetitionQueue>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PasswordPackage {
    #[prost(enumeration = "JudgeScheme", tag = "1")]
    pub scheme: i32,
    #[prost(message, repeated, tag = "2")]
    pub passwords: ::prost::alloc::vec::Vec<password_package::Pack>,
}
/// Nested message and enum types in `PasswordPackage`.
pub mod password_package {
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Pack {
        #[prost(string, tag = "1")]
        pub mark: ::prost::alloc::string::String,
        #[prost(message, repeated, tag = "2")]
        pub logins: ::prost::alloc::vec::Vec<super::super::auth::AuthRequest>,
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeclareCompetitionResult {
    #[prost(message, optional, tag = "1")]
    pub result: ::core::option::Option<super::generic::IdResult>,
    #[prost(message, optional, tag = "2")]
    pub staff: ::core::option::Option<PasswordPackage>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModCompDeclarationRequest {
    #[prost(int32, tag = "1")]
    pub comp_id: i32,
    #[prost(oneof = "mod_comp_declaration_request::Command", tags = "2, 3")]
    pub command: ::core::option::Option<mod_comp_declaration_request::Command>,
}
/// Nested message and enum types in `ModCompDeclarationRequest`.
pub mod mod_comp_declaration_request {
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Command {
        #[prost(message, tag = "2")]
        Redeclare(super::CompDeclaration),
        #[prost(enumeration = "super::ModDeclarationCommand", tag = "3")]
        SingleCommand(i32),
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompsStatusMessage {
    #[prost(message, repeated, tag = "1")]
    pub statuses: ::prost::alloc::vec::Vec<comps_status_message::CompStatusPair>,
}
/// Nested message and enum types in `CompsStatusMessage`.
pub mod comps_status_message {
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, Copy, PartialEq, ::prost::Message)]
    pub struct CompStatusPair {
        #[prost(int32, tag = "1")]
        pub comp_id: i32,
        #[prost(enumeration = "super::CompStatus", tag = "2")]
        pub status: i32,
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompsList {
    #[prost(map = "int32, message", tag = "1")]
    pub comp_views: ::std::collections::HashMap<i32, comps_list::CompView>,
}
/// Nested message and enum types in `CompsList`.
pub mod comps_list {
    #[derive(serde::Deserialize, serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CompView {
        #[prost(int32, tag = "1")]
        pub id: i32,
        #[prost(message, optional, tag = "2")]
        pub declaration: ::core::option::Option<super::CompDeclaration>,
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ModDeclarationCommand {
    Delete = 0,
    RemakeTempPwds = 1,
}
impl ModDeclarationCommand {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ModDeclarationCommand::Delete => "Delete",
            ModDeclarationCommand::RemakeTempPwds => "Remake_temp_pwds",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Delete" => Some(Self::Delete),
            "Remake_temp_pwds" => Some(Self::RemakeTempPwds),
            _ => None,
        }
    }
}
/// Comp Lifestatus:
/// Declaration -> Waiting -> Registration -> Waiting -> Running -> Completed
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CompStatus {
    /// Status when created, used until owner change it
    Declaration = 0,
    /// Team Registration process
    Registration = 1,
    /// Frozen status, just waiting.
    Waiting = 2,
    /// Comp is running
    Running = 3,
    /// Completed (Archived)
    Completed = 4,
    /// UNKNOWN or no Access
    Unknown = 5,
}
impl CompStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CompStatus::Declaration => "Declaration",
            CompStatus::Registration => "Registration",
            CompStatus::Waiting => "Waiting",
            CompStatus::Running => "Running",
            CompStatus::Completed => "Completed",
            CompStatus::Unknown => "Unknown",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Declaration" => Some(Self::Declaration),
            "Registration" => Some(Self::Registration),
            "Waiting" => Some(Self::Waiting),
            "Running" => Some(Self::Running),
            "Completed" => Some(Self::Completed),
            "Unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum JudgeScheme {
    FourFourOne = 0,
    FourFourTwo = 1,
    SixSixTwo = 2,
}
impl JudgeScheme {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            JudgeScheme::FourFourOne => "FourFourOne",
            JudgeScheme::FourFourTwo => "FourFourTwo",
            JudgeScheme::SixSixTwo => "SixSixTwo",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FourFourOne" => Some(Self::FourFourOne),
            "FourFourTwo" => Some(Self::FourFourTwo),
            "SixSixTwo" => Some(Self::SixSixTwo),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod competition_declarator_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct CompetitionDeclaratorClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl<T> CompetitionDeclaratorClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> CompetitionDeclaratorClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            CompetitionDeclaratorClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Создать описание соревнования
        pub async fn declare_competition(
            &mut self,
            request: impl tonic::IntoRequest<super::CompDeclaration>,
        ) -> std::result::Result<
            tonic::Response<super::DeclareCompetitionResult>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/comp.CompetitionDeclarator/DeclareCompetition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("comp.CompetitionDeclarator", "DeclareCompetition"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Изменить описание
        pub async fn modify_competition(
            &mut self,
            request: impl tonic::IntoRequest<super::ModCompDeclarationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::generic::GenericResultMessage>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/comp.CompetitionDeclarator/ModifyCompetition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("comp.CompetitionDeclarator", "ModifyCompetition"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn remake_staff_passwords(
            &mut self,
            request: impl tonic::IntoRequest<super::super::generic::Id>,
        ) -> std::result::Result<
            tonic::Response<super::PasswordPackage>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/comp.CompetitionDeclarator/RemakeStaffPasswords",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("comp.CompetitionDeclarator", "RemakeStaffPasswords"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_comps_status(
            &mut self,
            request: impl tonic::IntoRequest<super::super::generic::IdsList>,
        ) -> std::result::Result<
            tonic::Response<super::CompsStatusMessage>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/comp.CompetitionDeclarator/GetCompsStatus",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("comp.CompetitionDeclarator", "GetCompsStatus"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_comps_ids(
            &mut self,
            request: impl tonic::IntoRequest<super::super::generic::Empty>,
        ) -> std::result::Result<
            tonic::Response<super::super::generic::IdsList>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/comp.CompetitionDeclarator/GetCompsIds",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("comp.CompetitionDeclarator", "GetCompsIds"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_comps_views(
            &mut self,
            request: impl tonic::IntoRequest<super::super::generic::IdsList>,
        ) -> std::result::Result<tonic::Response<super::CompsList>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/comp.CompetitionDeclarator/GetCompsViews",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("comp.CompetitionDeclarator", "GetCompsViews"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_comp_declaration(
            &mut self,
            request: impl tonic::IntoRequest<super::super::generic::Id>,
        ) -> std::result::Result<
            tonic::Response<super::CompDeclaration>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/comp.CompetitionDeclarator/GetCompDeclaration",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("comp.CompetitionDeclarator", "GetCompDeclaration"),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
