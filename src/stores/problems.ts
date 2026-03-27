import { writable } from 'svelte/store';

export interface ProblemCounts {
  errors: number;
  warnings: number;
  infos: number;
}

export const problemCountsStore = writable<ProblemCounts>({
  errors: 0,
  warnings: 0,
  infos: 0,
});
