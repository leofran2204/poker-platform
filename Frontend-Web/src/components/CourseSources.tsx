import references from "@/data/courseSources.json";

export function CourseSources({ sourceIds }: { sourceIds?: string[] }) {
  const sources = (sourceIds ?? []).flatMap(id => {
    const source = references[id as keyof typeof references];
    return source ? [source] : [];
  });
  if (!sources.length) return null;
  return <details className="mt-5 border-t border-felt-700 pt-4">
    <summary className="cursor-pointer text-sm font-semibold text-gold-soft">Fontes e leituras · {sources.length}</summary>
    <ul className="mt-3 space-y-4">
      {sources.map(source => <li key={source.url} className="text-xs leading-relaxed">
        <a href={source.url} target="_blank" rel="noreferrer" className="font-semibold text-gold-soft underline underline-offset-4">{source.title} <span aria-label="abre em nova aba">↗</span></a>
        <p className="mt-1 text-felt-200">{source.credit}</p>
        <p className="mt-1 text-felt-300">{source.note}</p>
      </li>)}
    </ul>
  </details>;
}
