use std::path::PathBuf;

pub async fn is_process_running(process_name: &String) -> Result<(bool, Option<PathBuf>), String> {
    let process_name = process_name.clone();

    // TODO keep one system cached in process loop
    tokio::task::spawn_blocking(move || {
        // let pk = sysinfo::ProcessRefreshKind::nothing();
        let pk = sysinfo::ProcessRefreshKind::nothing()
            .with_exe(sysinfo::UpdateKind::OnlyIfNotSet)
            .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet);
        let k = sysinfo::RefreshKind::nothing().with_processes(pk);
        let mut sys = sysinfo::System::new_with_specifics(k);

        // TODO store PID and only update that pid
        // FIXME
        sys.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, true, pk);
        // sys.refresh_specifics(k);

        let res = sys.processes().values().find(|process| {
            if process
                .name()
                .to_string_lossy()
                .eq_ignore_ascii_case(&process_name)
            {
                return true;
            }
            // Under Proton /proc/PID/exe often points to wine-preloader rather
            // than the Windows executable. Check argv basenames, never substrings.
            #[cfg(target_os = "linux")]
            return process
                .cmd()
                .iter()
                .any(|arg| matches_game_argument(&arg.to_string_lossy(), &process_name));
            #[cfg(not(target_os = "linux"))]
            false
        });

        // dbg!(res);

        #[cfg(target_os = "windows")]
        let path = res
            .and_then(|first_match| first_match.exe())
            .map(|path| path.to_path_buf());

        // Do not suggest adding wine-preloader as an Overwatch installation.
        #[cfg(target_os = "linux")]
        let path = None;

        Ok((res.is_some(), path))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(target_os = "linux")]
fn matches_game_argument(argument: &str, name: &str) -> bool {
    argument
        .trim_matches('"')
        .rsplit(['/', '\\'])
        .next()
        .is_some_and(|basename| basename.eq_ignore_ascii_case(name))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    #[test]
    fn recognizes_proton_and_wine_without_matching_wrappers() {
        for argument in [
            "Overwatch.exe",
            "/mnt/Steam games/Overwatch/Overwatch.exe",
            "C:\\Games\\Overwatch.exe",
        ] {
            assert!(super::matches_game_argument(argument, "Overwatch.exe"));
        }
        for argument in [
            "/scripts/launch-Overwatch.exe.sh",
            "--game=Overwatch.exe",
            "/usr/bin/wine64",
        ] {
            assert!(!super::matches_game_argument(argument, "Overwatch.exe"));
        }
    }
}
