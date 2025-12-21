import type { Update } from "@tauri-apps/plugin-updater"

interface UpdateState {
    update: Update | null;
}

export const updateState : UpdateState = $state({
    update: null,
});