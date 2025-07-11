import { writable, get } from 'svelte/store';
import { api } from '../lib/api';

function createDocumentStore() {
  const store = writable({
    path: null as string | null,
    data: null as any,
    processing: false
  });

  const { subscribe, update } = store;

  return {
    subscribe,
    async select() {
      try {
        const response = await api.selectDocument();
        if (response?.path) {
          update(state => ({ ...state, path: response.path }));
        }
      } catch (error) {
        console.error('🐹 Error in document.select():', error);
      }
    },
    async process(options: any) {
      try {
        update(state => ({ ...state, processing: true }));
        const currentState = get(store);
        if (!currentState.path) {
          update(state => ({ ...state, processing: false }));
          return;
        }

        const data = await api.processDocument(currentState.path, options);
        console.log('🐹 Document processing result:', data);
        console.log('🐹 Data structure:', {
          hasData: !!data,
          dataKeys: data ? Object.keys(data) : 'no data',
          hasTables: data?.data?.tables ? `${data.data.tables.length} tables` : 'no tables',
          hasHtml: data?.data?.formatted_html ? 'has HTML' : 'no HTML',
          fullData: data
        });
        update(state => ({ ...state, data, processing: false }));
      } catch (error) {
        console.error('🐹 Error in document.process():', error);
        update(state => ({ ...state, processing: false }));
      }
    },
    async save() {
      // TODO: Implement save functionality
      console.log('🐹 Save functionality not yet implemented');
    }
  };
}

export const document = createDocumentStore();
