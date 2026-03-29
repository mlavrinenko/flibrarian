import type { Translation } from "./types";

const pluralRules = new Intl.PluralRules("ru");

function plural(
  count: number,
  forms: Partial<Record<Intl.LDMLPluralRule, string>> & { other: string },
): string {
  return forms[pluralRules.select(count)] ?? forms.other;
}

const ru: Translation = {
  search: {
    searchLabel: "Поиск",
    searchPlaceholder: "Искать книги...",
  },
  bookList: {
    noResults: "Ничего не найдено.",
    ariaLabel: "Результаты поиска",
    columns: {
      id: "ID",
      title: "Название",
      authors: "Авторы",
      genres: "Жанры",
      date: "Дата",
      lang: "Язык",
      fileSize: "Размер",
      sequence: "Серия",
      score: "Ранг",
    },
    filter: {
      placeholder: "Фильтр...",
      clearFilter: "Очистить",
      resizeColumn: "Изменить ширину колонки",
      notSpecified: "Не указан",
    },
  },
  bookCard: {
    anonymous: "Аноним",
  },
  indexing: {
    label: "Индексация:",
    modeAriaLabel: "Режим индексации",
    modeNew: "новое",
    modeFull: "всё",
    modeSearch: "поиск",
    modePick: "выбор",
    tipNew: "Индексировать только новые и прерванные архивы",
    tipFull: "Переразобрать и переиндексировать все архивы с нуля",
    tipSearch: "Пересоздать поисковый и FTS индексы без повторного разбора",
    tipPick: "Выбрать конкретные архивы для переиндексации",
    pickerTitle: "Выберите архивы для переиндексации",
    pickerReindex: "Переиндексировать",
    pickerSelectAll: "Выбрать все",
    pickerDeselectAll: "Снять все",
    pickerLoading: "Загрузка архивов...",
    pickerEmpty: "Архивы не найдены",
    progressAriaLabel: "Прогресс индексации",
    progressTooltip: "{phase}: {current} / {total}",
    inProgress: "Идёт индексация...",
    cancelConfirm: "Остановить индексацию?",
    cancel: "Остановить",
    closeConfirm:
      "Индексация ещё идёт. Закрыть приложение? Прогресс будет потерян.",
    phaseCounting: "Подсчёт книг",
    phaseParsing: "Разбор книг",
    phaseWriting: "Запись в базу данных",
    phaseBuildingSearchIndex: "Построение поискового индекса",
    phaseCreatingFtsIndex: "Создание FTS индекса",
  },
  bookDetail: {
    id: "ID",
    score: "Ранг",
    annotation: "Аннотация",
  },
  settings: {
    title: "Настройки",
    libraryPathLabel: "Путь к библиотеке",
    libraryPathPlaceholder: "/путь/к/библиотеке",
    defaultSaveFolderLabel: "Папка сохранения по умолчанию",
    defaultSaveFolderPlaceholder: "/путь/к/папке",
    browseButton: "Обзор...",
    saveButton: "Сохранить",
    cancelButton: "Отмена",
    saveFolderRequired: "Укажите папку для сохранения",
  },
  basket: {
    title: "Корзина",
    empty: "Корзина пуста",
    addToBasket: "В корзину",
    removeFromBasket: "Убрать",
    clearBasket: "Очистить",
    downloadAll: "Скачать все",
    downloading: "Скачивание...",
    downloadBook: "Скачать",
    downloadHint: "Ctrl+клик — скачать сразу",
    downloadSuccess: (count: number) =>
      count === 1 ? "Книга сохранена" : `${count} книг сохранено`,
    itemCount: (count: number) => `${count} кн.`,
  },
  logs: {
    title: "Логи",
    filterPlaceholder: "Фильтр логов...",
    clear: "Очистить",
    empty: "Нет записей.",
    levelInfo: "инфо+",
    levelWarn: "важные+",
    levelError: "ошибки",
    sourceIndexing: "индексация",
    sourceJs: "js",
    sourceApp: "приложение",
  },
  header: {
    keepsBooks: (count: number) =>
      `хранит ${count.toLocaleString("ru-RU")} ${plural(count, { one: "книгу", few: "книги", many: "книг", other: "книг" })}`,
    undo: "Отменить изменение фильтра",
    redo: "Повторить изменение фильтра",
  },
  themeCustomization: {
    title: "Тема",
    light: "Светлая",
    dark: "Тёмная",
    custom: "Своя",
    colorBg: "Фон",
    colorBgHover: "Наведение",
    colorBgSelected: "Выделение",
    colorBgInput: "Поле ввода",
    colorBgHeader: "Шапка",
    colorText: "Текст",
    colorTextSecondary: "Вторичный текст",
    colorBorder: "Рамка",
    colorPrimary: "Акцент",
    colorError: "Ошибка",
    resetDefaults: "Сбросить",
  },
  close: "Закрыть",
  dismiss: "Скрыть",
  languageSwitcher: "Язык",
  themeSwitcher: "Тема",
};

export default ru;
