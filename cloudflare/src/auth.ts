import { env } from 'cloudflare:workers';
import type { AuthRequest, OAuthHelpers } from '@cloudflare/workers-oauth-provider';
import { Hono } from 'hono';
import { Octokit } from 'octokit';
import { fetchUpstreamAuthToken, getUpstreamAuthorizeUrl, type Props } from './utils';
import {
  clientIdAlreadyApproved,
  parseRedirectApproval,
  renderApprovalDialog,
} from './workers-oauth-utils';

const app = new Hono<{ Bindings: Env & { OAUTH_PROVIDER: OAuthHelpers } }>();

app.get('/authorize', async (c) => {
  const oauthReqInfo = await c.env.OAUTH_PROVIDER.parseAuthRequest(c.req.raw);
  const { clientId } = oauthReqInfo;
  if (!clientId) {
    return c.text('Invalid request', 400);
  }

  if (
    await clientIdAlreadyApproved(c.req.raw, oauthReqInfo.clientId, env.COOKIE_ENCRYPTION_KEY)
  ) {
    return redirectToGithub(c.req.raw, oauthReqInfo);
  }

  return renderApprovalDialog(c.req.raw, {
    client: await c.env.OAUTH_PROVIDER.lookupClient(clientId),
    server: {
      description: 'Remote MCP server for displaying native popup dialogs on connected devices.',
      logo: 'https://avatars.githubusercontent.com/u/314135?s=200&v=4',
      name: 'Popup Service via GitHub',
    },
    state: { oauthReqInfo },
  });
});

app.post('/authorize', async (c) => {
  const { state, headers } = await parseRedirectApproval(c.req.raw, env.COOKIE_ENCRYPTION_KEY);
  if (!state.oauthReqInfo) {
    return c.text('Invalid request', 400);
  }

  return redirectToGithub(c.req.raw, state.oauthReqInfo, headers);
});

async function redirectToGithub(
  request: Request,
  oauthReqInfo: AuthRequest,
  headers: Record<string, string> = {}
) {
  return new Response(null, {
    headers: {
      ...headers,
      location: getUpstreamAuthorizeUrl({
        client_id: env.GITHUB_CLIENT_ID,
        redirect_uri: new URL('/callback', request.url).href,
        scope: 'read:user',
        state: btoa(JSON.stringify(oauthReqInfo)),
        upstream_url: 'https://github.com/login/oauth/authorize',
      }),
    },
    status: 302,
  });
}

/**
 * OAuth Callback Endpoint
 *
 * Handles GitHub callback, exchanges code for token, fetches user info
 */
app.get('/callback', async (c) => {
  const oauthReqInfo = JSON.parse(atob(c.req.query('state') as string)) as AuthRequest;
  if (!oauthReqInfo.clientId) {
    return c.text('Invalid state', 400);
  }

  const [accessToken, errResponse] = await fetchUpstreamAuthToken({
    client_id: c.env.GITHUB_CLIENT_ID,
    client_secret: c.env.GITHUB_CLIENT_SECRET,
    code: c.req.query('code'),
    redirect_uri: new URL('/callback', c.req.url).href,
    upstream_url: 'https://github.com/login/oauth/access_token',
  });
  if (errResponse) return errResponse;

  const user = await new Octokit({ auth: accessToken }).rest.users.getAuthenticated();
  const { login, name, email } = user.data;

  const { redirectTo } = await c.env.OAUTH_PROVIDER.completeAuthorization({
    metadata: {
      label: name || login,
    },
    props: {
      accessToken,
      email: email || '',
      login,
      name: name || login,
    } as Props,
    request: oauthReqInfo,
    scope: oauthReqInfo.scope,
    userId: login,
  });

  return Response.redirect(redirectTo);
});

export { app as GitHubHandler };
