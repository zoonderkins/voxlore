import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import en from "./locales/en/common.json";
import zhTW from "./locales/zh-TW/common.json";
import zhCN from "./locales/zh-CN/common.json";
import ja from "./locales/ja/common.json";

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    "zh-TW": { translation: zhTW },
    "zh-CN": { translation: zhCN },
    ja: { translation: ja },
  },
  lng: "en",
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
