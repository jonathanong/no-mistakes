use super::{EdgeKind, VitestSetupField};

pub(super) const fn key(kind: EdgeKind) -> (u8, u8) {
    match kind {
        EdgeKind::Import
        | EdgeKind::TypeImport
        | EdgeKind::DynamicImport
        | EdgeKind::RouteImport
        | EdgeKind::Require
        | EdgeKind::TestOf
        | EdgeKind::RouteRef
        | EdgeKind::QueueEnqueue
        | EdgeKind::QueueWorker
        | EdgeKind::RouteTest
        | EdgeKind::Layout
        | EdgeKind::MarkdownLink
        | EdgeKind::WorkspaceImport
        | EdgeKind::PackageDependency
        | EdgeKind::CiInvocation => core(kind),
        EdgeKind::HttpCall
        | EdgeKind::ProcessSpawn
        | EdgeKind::AssetImport
        | EdgeKind::Resource
        | EdgeKind::ReactRender
        | EdgeKind::Selector => runtime(kind),
        EdgeKind::SwiftImport
        | EdgeKind::SwiftReference
        | EdgeKind::SwiftPackageDependency
        | EdgeKind::DotnetUsing
        | EdgeKind::DotnetReference
        | EdgeKind::DotnetProjectDependency
        | EdgeKind::TerraformReference
        | EdgeKind::TerraformModuleRef
        | EdgeKind::TerraformOutputRef => language(kind),
        EdgeKind::WorkflowJob
        | EdgeKind::WorkflowStep
        | EdgeKind::WorkflowNeeds
        | EdgeKind::WorkflowUses
        | EdgeKind::WorkflowRun
        | EdgeKind::WorkflowArtifact
        | EdgeKind::VitestSetup(_) => workflow(kind),
    }
}

const fn core(kind: EdgeKind) -> (u8, u8) {
    match kind {
        EdgeKind::Import => (0, 0),
        EdgeKind::TypeImport => (1, 0),
        EdgeKind::DynamicImport => (2, 0),
        EdgeKind::RouteImport => (3, 0),
        EdgeKind::Require => (4, 0),
        EdgeKind::TestOf => (5, 0),
        EdgeKind::RouteRef => (6, 0),
        EdgeKind::QueueEnqueue => (7, 0),
        EdgeKind::QueueWorker => (8, 0),
        EdgeKind::RouteTest => (9, 0),
        EdgeKind::Layout => (10, 0),
        EdgeKind::MarkdownLink => (11, 0),
        EdgeKind::WorkspaceImport => (12, 0),
        EdgeKind::PackageDependency => (13, 0),
        EdgeKind::CiInvocation => (14, 0),
        _ => panic!("core edge group is exhaustive"),
    }
}

const fn runtime(kind: EdgeKind) -> (u8, u8) {
    match kind {
        EdgeKind::HttpCall => (15, 0),
        EdgeKind::ProcessSpawn => (16, 0),
        EdgeKind::AssetImport => (17, 0),
        EdgeKind::Resource => (18, 0),
        EdgeKind::ReactRender => (19, 0),
        EdgeKind::Selector => (20, 0),
        _ => panic!("runtime edge group is exhaustive"),
    }
}

const fn language(kind: EdgeKind) -> (u8, u8) {
    match kind {
        EdgeKind::SwiftImport => (21, 0),
        EdgeKind::SwiftReference => (22, 0),
        EdgeKind::SwiftPackageDependency => (23, 0),
        EdgeKind::DotnetUsing => (24, 0),
        EdgeKind::DotnetReference => (25, 0),
        EdgeKind::DotnetProjectDependency => (26, 0),
        EdgeKind::TerraformReference => (27, 0),
        EdgeKind::TerraformModuleRef => (28, 0),
        EdgeKind::TerraformOutputRef => (29, 0),
        _ => panic!("language edge group is exhaustive"),
    }
}

const fn workflow(kind: EdgeKind) -> (u8, u8) {
    match kind {
        EdgeKind::WorkflowJob => (30, 0),
        EdgeKind::WorkflowStep => (31, 0),
        EdgeKind::WorkflowNeeds => (32, 0),
        EdgeKind::WorkflowUses => (33, 0),
        EdgeKind::WorkflowRun => (34, 0),
        EdgeKind::WorkflowArtifact => (35, 0),
        EdgeKind::VitestSetup(VitestSetupField::SetupFiles) => (36, 0),
        EdgeKind::VitestSetup(VitestSetupField::GlobalSetup) => (36, 1),
        _ => panic!("workflow edge group is exhaustive"),
    }
}
