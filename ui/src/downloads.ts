interface DownloadRecord {
  id: number;
  url: string;
  file_path: string | null;
  status: 'downloading' | 'complete' | 'failed' | 'interrupted';
  started_ms: number;
}

(function () {
  const stateElement = document.getElementById('titan-downloads-state');
  const downloads: DownloadRecord[] = stateElement?.textContent
    ? JSON.parse(stateElement.textContent) as DownloadRecord[]
    : [];
  const list = document.getElementById('downloadList') as HTMLElement;
  const clearButton = document.getElementById('clearDownloads') as HTMLButtonElement;

  function send(message: object): void {
    window.ipc?.postMessage(JSON.stringify(message));
  }

  function fileName(download: DownloadRecord): string {
    const source = download.file_path || new URL(download.url).pathname;
    return source.split(/[\\/]/).filter(Boolean).pop() || 'Download';
  }

  if (downloads.length === 0) {
    const empty = document.createElement('div');
    empty.id = 'empty';
    empty.textContent = 'Downloaded files will appear here';
    list.append(empty);
  } else {
    for (const download of downloads) {
      const row = document.createElement('div');
      row.className = 'entry';
      const info = document.createElement('div');
      const name = document.createElement('div');
      name.className = 'name';
      name.textContent = fileName(download);
      const detail = document.createElement('div');
      detail.className = `detail status-${download.status}`;
      detail.textContent = `${download.status} · ${new Date(download.started_ms).toLocaleString()}`;
      info.append(name, detail);

      const open = document.createElement('button');
      open.type = 'button';
      open.textContent = download.status === 'complete' ? 'Open' : download.status;
      open.disabled = download.status !== 'complete' || !download.file_path;
      open.addEventListener('click', () => send({ type: 'OpenDownload', download_id: download.id }));
      row.append(info, open);
      list.append(row);
    }
  }

  clearButton.disabled = downloads.length === 0;
  clearButton.addEventListener('click', () => send({ type: 'ClearDownloads' }));
})();
