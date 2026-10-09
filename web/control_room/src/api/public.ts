// The public routes (FRONTEND.md 7) with their cache topics, shared by the View page, the news bar,
// the intro and the admin overview.
import { useApi, prefetch, type Topic } from './cache';
import type { Brief, Dam, Fires, RegionOverview } from './types';

export const PUBLIC = {
  overview: { path: '/region/overview', topics: ['zones'] as Topic[] },
  fires: { path: '/fires?hours=72', topics: ['fires'] as Topic[] },
  dams: { path: '/dams', topics: ['dams'] as Topic[] },
  brief: { path: '/briefs/latest?scope=region', topics: ['briefs'] as Topic[] },
};

export const useOverview = () => useApi<RegionOverview>(PUBLIC.overview.path, PUBLIC.overview.topics);
export const useFires = () => useApi<Fires>(PUBLIC.fires.path, PUBLIC.fires.topics);
export const useDams = () => useApi<{ dams: Dam[] }>(PUBLIC.dams.path, PUBLIC.dams.topics);
export const useBrief = () => useApi<{ brief: Brief | null }>(PUBLIC.brief.path, PUBLIC.brief.topics);

/** The first-visit intro waits for these; each step reports when it is in the cache. */
export const PRELOAD = Object.values(PUBLIC).map(p => () => prefetch(p.path, p.topics));
