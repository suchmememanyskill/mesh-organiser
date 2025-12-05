export interface AccountLinkData {
    showLinkUi: boolean;
    baseUrl: string;
    userName: string;
    linkToken: string;
}

export const accountLinkData = $state<AccountLinkData>({
    showLinkUi: false,
    baseUrl: '',
    userName: '',
    linkToken: '',
});