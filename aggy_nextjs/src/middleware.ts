import csrf from 'edge-csrf';
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';


export async function middleware(request: NextRequest) {
  const response = NextResponse.next();

  // csrf protection
  const csrfError = await csrfProtect(request, response);

  // check result
  if (csrfError) {
    return new NextResponse('invalid csrf token', { status: 403 });
  }

  return response;
}

// initalize protection function
const csrfProtect = csrf({
  cookie: {
    secure: process.env.NODE_ENV === 'production',
  },
  token: {
    value: async (request) => {
      // check the `x-csrf-token` request header
      let token = request.headers.get('x-csrf-token');
      if (token !== null) return token;

      // check request body
      const contentType = request.headers.get('content-type') || 'text/plain';

      if (
        /multipart\/form-data/.test(contentType)
      ) {
        const formData = await request.formData();
        const formDataVal = formData.get('1_csrf_token')
        if (typeof formDataVal === 'string') return formDataVal
        console.log(
          'CSRF token missing for form submission',
          { contentType, formData: [...formData.entries()] }
        );
        return ''
      }
      // url-encoded
      if (
        contentType === 'application/x-www-form-urlencoded'
      ) {
        const formData = await request.formData();
        const formDataVal = formData.get('csrf_token')
        if (typeof formDataVal === 'string') return formDataVal
        console.log({ contentType, testWorks: /multipart\/form-data/.test(contentType), formData: [...formData.entries()] });
        console.log(
          'CSRF token missing for urlencoded form submission',
          { contentType, formData: [...formData.entries()] }
        );
        return ''
      }

      // json-encoded
      if (contentType === 'application/json' ||
        contentType === 'application/ld+json') {
        const json = await request.json();
        const jsonVal = json['csrf_token']
        if (typeof jsonVal === 'string') return jsonVal
        return ''
      }

      return await request.text();
    }
  }
});
