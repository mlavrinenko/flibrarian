class CoverViewerState {
  open = $state(false);
  url: string | null = $state(null);
  title = $state("");

  show(url: string, title: string) {
    this.url = url;
    this.title = title;
    this.open = true;
  }

  close() {
    this.open = false;
    this.url = null;
    this.title = "";
  }
}

export const coverViewer = new CoverViewerState();
