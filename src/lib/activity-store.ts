import { writable } from 'svelte/store';

// Built-in activity IDs
export type BuiltInActivityId = 'explorer' | 'search' | 'scm' | 'debug' | 'extensions';

// Activity ID can be built-in or extension-provided (e.g., "docker-explorer")
export type ActivityId = BuiltInActivityId | string;

export const activeActivity = writable<ActivityId>('explorer');
