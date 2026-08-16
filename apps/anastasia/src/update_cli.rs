//! `anastasia update` — check for and apply a newer release, natively (the same
//! flow `edge/src/install.sh` performs: download → verify → symlink swap →
//! service restart). macOS app bundles swap the bundle instead; source builds
//! are report-only.

use anastasia_update::{InstallKind, current_version, version_newer};
use anyhow::bail;

/// `--check` prints the verdict and exits (nonzero when an update is available,
/// so scripts can gate on it).
pub async fn update(edge_url: &str, check_only: bool) -> anyhow::Result<()> {
    let manifest = anastasia_update::fetch_latest(edge_url).await?;
    let current = current_version();
    if !version_newer(&manifest.version, current) {
        println!(
            "anastasia {current} is up to date (latest: {}).",
            manifest.version
        );
        return Ok(());
    }
    println!("anastasia {current} → {} available", manifest.version);
    if check_only {
        std::process::exit(1);
    }

    match anastasia_update::detect_install() {
        InstallKind::Managed { app_root } => {
            println!(
                "downloading {}…",
                anastasia_update::headless_artifact(&manifest.version)
            );
            anastasia_update::stage_headless(edge_url, &manifest, &app_root).await?;
            anastasia_update::apply_headless(&app_root, &manifest.version)?;
            println!(
                "installed {} (current → {})",
                app_root.join(&manifest.version).display(),
                manifest.version
            );
            match anastasia_update::restart_service() {
                Ok(()) => println!("engine service restarted."),
                Err(err) => println!(
                    "note: service restart failed ({err:#}) — restart the engine manually to finish."
                ),
            }
            Ok(())
        }
        InstallKind::MacApp { bundle } => {
            println!(
                "downloading {}…",
                anastasia_update::mac_app_artifact(&manifest.version)
            );
            let data_dir = std::env::var_os("ANASTASIA_DATA_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(super::dirs_data_dir);
            let staged = anastasia_update::stage_mac_app(edge_url, &manifest, &data_dir).await?;
            anastasia_update::apply_mac_app(&staged, &bundle)?;
            println!(
                "updated {} — relaunch Anastasia to finish.",
                bundle.display()
            );
            Ok(())
        }
        InstallKind::Unmanaged => {
            bail!(
                "this binary is not update-managed (source build or hand-copied).\n\
                 Download the latest release from\n\
                 https://github.com/cowboyshibuya/anastasia/releases/latest\n\
                 or rebuild from source."
            )
        }
    }
}
