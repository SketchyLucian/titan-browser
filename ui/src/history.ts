interface HistoryEntry {
  title: string;
  url: string;
  last_visited_ms: number;
  visit_count: number;
}

(function () {
  const stateElement = document.getElementById('titan-history-state');
  const history: HistoryEntry[] = stateElement?.textContent
    ? JSON.parse(stateElement.textContent) as HistoryEntry[]
    : [];
  const list = document.getElementById('historyList') as HTMLElement;
  const search = document.getElementById('historySearch') as HTMLInputElement;
  const clearButton = document.getElementById('clearHistory') as HTMLButtonElement;

  function send(message: object): void {
    window.ipc?.postMessage(JSON.stringify(message));
  }

  function render(): void {
    const query = search.value.trim().toLowerCase();
    const entries = history.filter((entry) =>
      !query || entry.title.toLowerCase().includes(query) || entry.url.toLowerCase().includes(query)
    );
    list.replaceChildren();

    if (entries.length === 0) {
      const empty = document.createElement('div');
      empty.id = 'empty';
      empty.textContent = query ? 'No matching history' : 'No browsing history yet';
      list.append(empty);
      return;
    }

    for (const entry of entries) {
      const row = document.createElement('button');
      row.className = 'entry';
      row.type = 'button';
      row.addEventListener('click', () => send({ type: 'Navigate', url: entry.url }));

      const time = document.createElement('span');
      time.className = 'time';
      time.textContent = new Date(entry.last_visited_ms).toLocaleString();

      const page = document.createElement('span');
      const title = document.createElement('div');
      title.className = 'title';
      title.textContent = entry.title || entry.url;
      const url = document.createElement('div');
      url.className = 'url';
      url.textContent = entry.url;
      page.append(title, url);
      row.append(time, page);
      list.append(row);
    }
  }

  search.addEventListener('input', render);
  clearButton.addEventListener('click', () => {
    if (history.length === 0 || window.confirm('Clear all browsing history?')) {
      send({ type: 'ClearHistory' });
    }
  });
  render();
})();
