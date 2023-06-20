import * as Separator from '@radix-ui/react-separator';
import React from 'react';

export { Image, type ImageProps } from "@unpic/react";
export * as Toolbar from '@radix-ui/react-toolbar';
export * as Toggle from '@radix-ui/react-toggle';
export * as Tooltip from '@radix-ui/react-tooltip';
export * as RadixForm from '@radix-ui/react-form';

export const Divider = React.forwardRef<HTMLDivElement, Separator.SeparatorProps>(
  function Divider(props, ref) {
    return (
      <Separator.Root
        className="divider"
        ref={ref}
        {...props}
      />
    );
  }
)
