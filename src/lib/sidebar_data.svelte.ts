import { getContainer } from "./api/dependency_injection";
import { defaultSidebarState, ISidebarStateApi } from "./api/shared/services/sidebar_state_api";

const sidebarState = defaultSidebarState();

export async function updateSidebarState() : Promise<void> {
    let sidebarStateApi = getContainer().require<ISidebarStateApi>(ISidebarStateApi);
    let newState = await sidebarStateApi.getSidebarState();
    Object.assign(sidebarState, newState);
}