import type { EntryGenerator } from './$types';

export const entries: EntryGenerator = () => {
    return [
        { slug: '1' },
        { slug: '2' }
    ];
};

export const prerender = true;