import { getContainer } from "./api/dependency_injection";
import { configurationDefault, ISettingsApi, type Configuration } from "./api/shared/settings_api";
import { type User } from "./api/shared/user_api";
import { debounce } from "./utils";

// TODO: Change this to use the same structure as useSidebar()
export const configuration = $state(configurationDefault());
export const configurationMeta = $state({
    configurationLoaded: false,
    applicationReadOnly: false,
});
export const currentUser = $state<User>({
  id: -1,
  username: "",
  email: "",
  created: new Date(),
  permissions: {
    admin: false,
    sync: false,
    onlineAccount: false,
  },
  syncUrl: null,
  syncToken: null,
  lastSync: null,
})

export async function updateConfiguration(config : Configuration) : Promise<void> {
    let settingsApi = getContainer().optional<ISettingsApi>(ISettingsApi);
    
    if (!settingsApi) {
        console.warn("No settings API available to save configuration");
        return;
    }

    await settingsApi.saveConfiguration(config);
}

/*
function rgbToHsl(r : number, g : number, b : number) : [number, number, number] {
  r /= 255, g /= 255, b /= 255;

  var max = Math.max(r, g, b), min = Math.min(r, g, b);
  var h, s, l = (max + min) / 2;

  if (max == min) {
    h = s = 0; // achromatic
  } else {
    var d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break;
      case g: h = (b - r) / d + 2; break;
      case b: h = (r - g) / d + 4; break;
      default: h = 0;
    }

    h /= 6;
  }

  return [ h, s, l ];
}

const thumbnailColorAsFilters = $derived.by(() => {
    const rgbTextColor = configuration.thumbnail_color;
    const r = parseInt(rgbTextColor.slice(1, 3), 16);
    const g = parseInt(rgbTextColor.slice(3, 5), 16);
    const b = parseInt(rgbTextColor.slice(5, 7), 16);

    const [h, s, l] = rgbToHsl(r, g, b);

    console.log(`hsl: ${h}, ${s}, ${l}`);

    return `hue-rotate(${Math.round(h * 360)}deg) saturate(${Math.round(s * 100)}%) brightness(${Math.round(l * 100)}%)`;
})

export function getThumbnailColorAsFilters() : string {
    return thumbnailColorAsFilters;
}
*/