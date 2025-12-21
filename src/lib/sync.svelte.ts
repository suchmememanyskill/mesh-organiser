export enum SyncStage {
    Idle,
    Models,
    Groups,
    Labels,
    Resources
};

export enum SyncStep {
    Init,
    Upload,
    Download,
    UpdateMetadata,
    Delete,
}

export interface SyncState {
    stage: SyncStage;
    step: SyncStep;
    processableItems: number;
    processedItems: number;
}

function defaultSyncState() : SyncState
{
    return {
        stage: SyncStage.Idle,
        step: SyncStep.Init,
        processableItems: 0,
        processedItems: 0,
    }
}

export const globalSyncState : SyncState = $state(defaultSyncState());

export function resetSyncState() : void
{
    Object.assign(globalSyncState, defaultSyncState());
}