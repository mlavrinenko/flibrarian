export interface Translation {
  search: {
    searchLabel: string;
    searchPlaceholder: string;
  };
  bookList: {
    noResults: string;
    ariaLabel: string;
    columns: {
      id: string;
      title: string;
      authors: string;
      genres: string;
      date: string;
      lang: string;
      fileSize: string;
      sequence: string;
      score: string;
    };
    filter: {
      placeholder: string;
      clearFilter: string;
      resizeColumn: string;
      notSpecified: string;
    };
  };
  bookCard: {
    anonymous: string;
  };
  indexing: {
    label: string;
    modeAriaLabel: string;
    modeNew: string;
    modeFull: string;
    modeSearch: string;
    modePick: string;
    tipNew: string;
    tipFull: string;
    tipSearch: string;
    tipPick: string;
    pickerTitle: string;
    pickerReindex: string;
    pickerSelectAll: string;
    pickerDeselectAll: string;
    pickerLoading: string;
    pickerEmpty: string;
    progressAriaLabel: string;
    progressTooltip: string;
    inProgress: string;
    cancelConfirm: string;
    cancel: string;
    closeConfirm: string;
    phaseCounting: string;
    phaseParsing: string;
    phaseWriting: string;
    phaseBuildingSearchIndex: string;
    phaseCreatingFtsIndex: string;
  };
  bookDetail: {
    id: string;
    score: string;
    annotation: string;
  };
  settings: {
    title: string;
    libraryPathLabel: string;
    libraryPathPlaceholder: string;
    defaultSaveFolderLabel: string;
    defaultSaveFolderPlaceholder: string;
    browseButton: string;
    saveButton: string;
    cancelButton: string;
    saveFolderRequired: string;
  };
  basket: {
    title: string;
    empty: string;
    addToBasket: string;
    removeFromBasket: string;
    clearBasket: string;
    downloadAll: string;
    downloading: string;
    downloadBook: string;
    downloadHint: string;
    downloadSuccess: (count: number) => string;
    itemCount: (count: number) => string;
  };
  logs: {
    title: string;
    filterPlaceholder: string;
    clear: string;
    empty: string;
    levelInfo: string;
    levelWarn: string;
    levelError: string;
    sourceIndexing: string;
    sourceJs: string;
    sourceApp: string;
  };
  header: {
    keepsBooks: (count: number) => string;
    undo: string;
    redo: string;
  };
  themeCustomization: {
    title: string;
    light: string;
    dark: string;
    custom: string;
    colorBg: string;
    colorBgHover: string;
    colorBgSelected: string;
    colorBgInput: string;
    colorBgHeader: string;
    colorText: string;
    colorTextSecondary: string;
    colorBorder: string;
    colorPrimary: string;
    colorError: string;
    resetDefaults: string;
  };
  close: string;
  dismiss: string;
  languageSwitcher: string;
  themeSwitcher: string;
}
