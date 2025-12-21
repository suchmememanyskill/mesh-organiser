import type { Blob } from "../shared/blob_api";
import { FileType, createBlobInstance } from "../shared/blob_api";
import type { Model } from "../shared/model_api";
import { createModelInstance } from "../shared/model_api";
import type { GroupMeta } from "../shared/group_api";
import { createGroupMetaInstance } from "../shared/group_api";
import type { LabelMeta } from "../shared/label_api";
import { createLabelMetaInstance } from "../shared/label_api";

// Helper function to determine file type from URL
function getFileTypeFromUrl(url: string): FileType {
    const lower = url.toLowerCase();
    if (lower.endsWith('.3mf')) return FileType.THREEMF;
    if (lower.endsWith('.gcode')) return FileType.GCODE;
    if (lower.endsWith('.obj')) return FileType.OBJ;
    if (lower.endsWith('.step')) return FileType.STEP;
    return FileType.STL; // default
}

// Helper function to extract filename from URL
function getFilenameFromUrl(url: string): string {
    const parts = url.split('/');
    const filename = parts[parts.length - 1];
    // Remove extension for display name
    return filename.replace(/\.(stl|3mf|gcode|obj|step)(\.png)?$/i, '');
}

// Helper function to parse size string to bytes
function parseSizeToBytes(size: string): number {
    const match = size.match(/^([\d.]+)\s*(B|KB|MB|GB)$/i);
    if (!match) return 0;
    
    const value = parseFloat(match[1]);
    const unit = match[2].toUpperCase();
    
    switch (unit) {
        case 'B': return Math.floor(value);
        case 'KB': return Math.floor(value * 1024);
        case 'MB': return Math.floor(value * 1024 * 1024);
        case 'GB': return Math.floor(value * 1024 * 1024 * 1024);
        default: return 0;
    }
}

// Mock model data
interface MockModelData {
    name: string;
    modelUrl: string;
    thumbnailUrl: string;
    size: string;
    sourceUrl?: string;
}

const mockModelDataList: MockModelData[] = [
    {
        name: "3DBenchy",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/3dbenchy.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/3dbenchy.stl.png",
        size: "10.8 MB",
        sourceUrl: "https://www.printables.com/model/2236-3dbenchy-the-jolly-3d-printing-torture-test-by-cre"
    },
    {
        name: "Calibration Cube",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/Calibration Cube.3mf",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/Calibration Cube.3mf.png",
        size: "4.5 KB",
        sourceUrl: "https://www.printables.com/model/118657-calibration-cube"
    },
    {
        name: "Embossed Text",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/Embossed text.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/Embossed text.stl.png",
        size: "69.2 KB"
    },
    {
        name: "Temp Tower PLA",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/Temp Tower PLA.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/Temp Tower PLA.stl.png",
        size: "2.1 MB",
        sourceUrl: "https://www.printables.com/model/274675-temp-tower-explanations"
    },
    {
        name: "Beer Crate AA",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/beer_crate_AA.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/beer_crate_AA.stl.png",
        size: "237.8 KB",
        sourceUrl: "https://www.printables.com/model/149774-beer-crate-battery-holder-aa-aaa-boxes"
    },
    {
        name: "Boaty",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/boaty.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/boaty.stl.png",
        size: "583.4 KB",
        sourceUrl: "https://www.printables.com/model/1141963-3d-boaty"
    },
    {
        name: "Eiffel Tower",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/eiffel_final.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/eiffel_final.stl.png",
        size: "7.7 MB",
        sourceUrl: "https://www.printables.com/model/572-eiffel-tower"
    },
    {
        name: "Calibration Cube (GCode)",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/ECC_0.4_Korper1_PLA0.2_12m19s.gcode",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/ECC_0.4_Korper1_PLA0.2_12m19s.gcode.png",
        size: "369.0 KB",
        sourceUrl: "https://www.printables.com/model/118657-calibration-cube"
    },
    {
        name: "Cone",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/Cone.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/Cone.stl.png",
        size: "17.8 KB"
    },
    {
        name: "Cube",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/Cube.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/Cube.stl.png",
        size: "684.0 B"
    },
    {
        name: "Cylinder",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/Cylinder.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/Cylinder.stl.png",
        size: "35.2 KB"
    },
    {
        name: "Disc",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/Disc.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/Disc.stl.png",
        size: "35.2 KB"
    },
    {
        name: "Sphere",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/Sphere.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/Sphere.stl.png",
        size: "1.5 MB"
    },
    {
        name: "Torus",
        modelUrl: "https://meshdemo.suchmeme.nl/mock_data/Torus.stl",
        thumbnailUrl: "https://meshdemo.suchmeme.nl/mock_data/Torus.stl.png",
        size: "1.4 MB"
    }
];

// Create blobs and models
let nextId = 1;
let nextBlobId = 1;

export const mockBlobs: Map<number, Blob> = new Map();
export const mockModels: Map<number, Model> = new Map();

// Generate mock data
const baseDate = new Date('2024-01-01');
mockModelDataList.forEach((data, index) => {
    const blob = createBlobInstance(
        nextBlobId++,
        `sha256_${index}_${getFilenameFromUrl(data.modelUrl)}`,
        getFileTypeFromUrl(data.modelUrl),
        parseSizeToBytes(data.size),
        new Date(baseDate.getTime() + index * 86400000).toISOString() // Add a day for each model
    );
    
    // Store model URL and thumbnail URL on blob for later use
    (blob as any)._modelUrl = data.modelUrl;
    (blob as any)._thumbnailUrl = data.thumbnailUrl;
    
    mockBlobs.set(blob.id, blob);

    let flags = [];

    if (Math.random() < 0.2) {
        flags.push("Printed")
    }

    if (Math.random() < 0.1) {
        flags.push("Favorite")
    }

    const model = createModelInstance(
        nextId++,
        data.name,
        blob,
        data.sourceUrl ?? null,
        null,
        blob.added.toISOString(),
        blob.added.toISOString(),
        null,
        [],
        flags,
        ""
    );
    
    mockModels.set(model.id, model);
});

// Create groups
export const mockGroups: Map<number, GroupMeta> = new Map();

const primitivesGroup = createGroupMetaInstance(
    1,
    "primitives",
    new Date('2024-01-15').toISOString(),
    new Date('2024-01-15').toISOString(),
    ""
);
mockGroups.set(primitivesGroup.id, primitivesGroup);

// Create labels
export const mockLabels: Map<number, LabelMeta> = new Map();

const benchmarkingLabel = createLabelMetaInstance(
    1,
    "benchmarking models",
    0x3b82f6, // blue color
    new Date().toISOString(),
    ""
);
mockLabels.set(benchmarkingLabel.id, benchmarkingLabel);

// Assign models to groups and labels
const modelsByName: Map<string, Model> = new Map();
mockModels.forEach(model => {
    modelsByName.set(model.name.toLowerCase(), model);
});

// Add primitives to group
const primitiveNames = ['sphere', 'cone', 'cube', 'cylinder', 'disc', 'torus'];
primitiveNames.forEach(name => {
    const model = modelsByName.get(name);
    if (model) {
        model.group = primitivesGroup;
    }
});

// Add benchmarking models to label
const benchmarkNames = ['boaty', '3dbenchy', 'eiffel tower'];
benchmarkNames.forEach(name => {
    const model = modelsByName.get(name);
    if (model) {
        model.labels = [benchmarkingLabel];
    }
});

// Track model-to-group and model-to-label relationships
export const modelGroupMap: Map<number, number> = new Map();
export const modelLabelsMap: Map<number, number[]> = new Map();

mockModels.forEach(model => {
    if (model.group) {
        modelGroupMap.set(model.id, model.group.id);
    }
    if (model.labels.length > 0) {
        modelLabelsMap.set(model.id, model.labels.map(l => l.id));
    }
});
