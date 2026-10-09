// The moving news bar. Lines come from Inbox > News bar. The animation is pure CSS (no script per frame)
// and pauses while the mouse is over it.
import { useMemo } from 'react';
import { db } from '../data/db';
import { useRows } from '../data/store';
import { useI18n } from '../i18n';

export function Ticker({ where }: { where: 'public' | 'admin' }) {
  const { t, b } = useI18n();
  const news = useRows(db.news);
  const items = useMemo(() => news.filter(n => n.active && (n.where === 'both' || n.where === where)).sort((a, z) => a.order - z.order), [news, where]);
  if (!items.length) return null;
  const line = items.map(n => <span className="mq-item" key={n.id}><span className="sep">●</span><bdi>{b(n.text)}</bdi></span>);
  return (
    <div className="ticker" role="region" aria-label={t('common.news')}>
      <span className="ticker-tag"><i className="live" />{t('common.news')}</span>
      <div className="ticker-win">
        <div className="mq-track mq-run" style={{ ['--mq-dur' as string]: Math.max(24, items.length * 9) + 's' }}>
          <span>{line}</span><span aria-hidden="true">{line}</span>
        </div>
      </div>
    </div>
  );
}
