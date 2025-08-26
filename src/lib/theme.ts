import { exists, writeTextFile, readTextFile, BaseDirectory } from '@tauri-apps/plugin-fs';
import cssText from "/src/themes/default.css?raw";

const availableThemes = {
    "default": "Default",
    "shadcnred": "Shadcn Red",
    "shadcnrose": "Shadcn Rose",
    "shadcnorange": "Shadcn Orange",
    "shadcngreen": "Shadcn Green",
    "shadcnblue": "Shadcn Blue",
    "shadcnyellow": "Shadcn Yellow",
    "shadcnviolet": "Shadcn Violet",
    "nord": "Nord",
    "vintageretro": "Vintage Retro",
    "synthwave": "Synthwave",
    "quantumrose": "Quantum Rose",
    "graphite": "Graphite",
    "perpetuity": "Perpetuity",
    "nature": "Nature",
    "sunsethorizon": "Sunset Horizon",
    "custom": "Custom",
}

export function getThemeName(theme : string) : string
{
    let index = theme as keyof typeof availableThemes;
    if (index in availableThemes)
    {
        return availableThemes[index];
    }

    return "(Unknown)";
}

export function getAvailableThemes() : string[]
{
    return Object.keys(availableThemes);
}

export async function setTheme(theme : string) 
{
    if (!(theme in availableThemes))
    {
        theme = "default";
    } 

    let existingCustomTheme = document.getElementById("custom-theme");
    if (existingCustomTheme)
    {
        existingCustomTheme.remove();
    }

    if (theme === "custom")
    {
        console.log("Loading custom theme");
        if (!await exists("custom.css", { baseDir: BaseDirectory.AppData }))
        {
            console.log("Custom theme does not exist, creating default");
            await writeTextFile('custom.css', cssText.replaceAll("[data-theme=\"default\"]", "[data-theme=\"custom\"]"), {
                baseDir: BaseDirectory.AppData,
            });
        }

        let custom_css = await readTextFile('custom.css', {
            baseDir: BaseDirectory.AppData,
        });

        let style = document.createElement('style');
        style.textContent = custom_css;
        style.id = "custom-theme";
        document.head.appendChild(style);
    }

    document.documentElement.setAttribute("data-theme", theme);
}