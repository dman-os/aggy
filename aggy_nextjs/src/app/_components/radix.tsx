"use client"

import * as RadF from '@radix-ui/react-form';
import React from 'react';

export const RadFRoot = React.forwardRef<HTMLFormElement, RadF.FormProps>(
  function RadFRoot(props, ref) {
    return <RadF.Root ref={ref} {...props} />
  }
);

export const RadFField = React.forwardRef<HTMLDivElement, RadF.FormFieldProps>(
  function RadFField(props, ref) {
    return <RadF.Field ref={ref} {...props} />
  }
);

export const RadFLabel = React.forwardRef<HTMLLabelElement, RadF.FormLabelProps>(
  function RadFLabel(props, ref) {
    return <RadF.Label ref={ref} {...props} />
  }
);
export const RadFControl = React.forwardRef<HTMLInputElement, RadF.FormControlProps>(
  function RadFControl(props, ref) {
    return <RadF.Control ref={ref} {...props} />
  }
);
export const RadFMessage = React.forwardRef<HTMLSpanElement, RadF.FormMessageProps>(
  function RadFMessage(props, ref) {
    return <RadF.Message ref={ref} {...props} />
  }
);
export const RadFSubmit = React.forwardRef<HTMLButtonElement, RadF.FormSubmitProps>(
  function RadFSubmit(props, ref) {
    return <RadF.Submit ref={ref} {...props} />
  }
);
