#[cfg(feature = "__turborepo")]
pub mod turborepo {
    #[cfg(feature = "__turborepo_path")]
    pub use turbopath as path;
    #[cfg(feature = "__turborepo_ci")]
    pub use turborepo_ci as ci;
    #[cfg(feature = "__turborepo_repository")]
    pub use turborepo_repository as repository;
}
