import { useI18n } from '../i18n';

export function LangSwitch() {
  const { lang, setLang } = useI18n();
  return (
    <div className="lang" role="group" aria-label="Language">
      <button className={'ku' + (lang === 'ku' ? ' on' : '')} onClick={() => setLang('ku')} lang="ckb">کوردی</button>
      <button className={lang === 'en' ? 'on' : ''} onClick={() => setLang('en')} lang="en">EN</button>
    </div>
  );
}
