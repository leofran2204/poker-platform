type BrandMarkProps = {
  size?: number;
  className?: string;
};

/** Símbolo oficial (ás). O nome “Zero Tilt” fica em tipo ao lado. */
export function BrandMark({ size = 28, className }: BrandMarkProps) {
  return (
    <img
      src="/brand/mark.jpg"
      alt=""
      width={size}
      height={size}
      className={className ?? "h-7 w-7 shrink-0 rounded-sm object-cover ring-1 ring-gold/40"}
      aria-hidden
    />
  );
}
