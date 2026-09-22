import { type InputHTMLAttributes, forwardRef } from 'react';
import { clsx } from 'clsx';

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, label, error, type = 'text', ...props }, ref) => {
    return (
      <div className="space-y-1.5">
        {label && (
          <label className="block text-sm font-medium text-text">
            {label}
          </label>
        )}
        <input
          ref={ref}
          type={type}
          className={clsx(
            'w-full px-3 py-2 text-sm bg-surface border rounded-lg',
            'transition-colors focus:outline-none',
            'placeholder:text-text-subtle',
            error 
              ? 'border-danger focus:border-danger focus:ring-1 focus:ring-danger'
              : 'border-border-strong focus:border-accent focus:ring-1 focus:ring-accent',
            className
          )}
          {...props}
        />
        {error && (
          <p className="text-xs text-danger">{error}</p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';
