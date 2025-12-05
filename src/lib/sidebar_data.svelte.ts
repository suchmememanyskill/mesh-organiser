import { getContainer } from "./api/dependency_injection";
import { defaultSidebarState, ISidebarStateApi } from "./api/shared/sidebar_state_api";

export const sidebarState = $state(defaultSidebarState());

export async function updateSidebarState() : Promise<void> {
    let sidebarStateApi = getContainer().optional<ISidebarStateApi>(ISidebarStateApi);
    if (!sidebarStateApi) {
        return;
    }
    
    let newState = await sidebarStateApi.getSidebarState();
    Object.assign(sidebarState, newState);
}