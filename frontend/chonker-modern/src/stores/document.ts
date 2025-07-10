import { writable } from 'svelte/store';

function createDocumentStore() {
  const store = writable({
    path: null as string | null
  });

  const { subscribe, update } = store;

  return {
    subscribe,
    setPath(path: string) {
      update(state => ({ ...state, path }));
    }
  };
}

export const document = createDocumentStore();
