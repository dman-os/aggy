import type { LoaderArgs, V2_MetaFunction } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";
import { newApiClient } from "~/api/index.server";
import { PostStatusLines } from "~/components";
import RootLayout from "~/components/layout";

export const meta: V2_MetaFunction = () => {
  return [
    { title: "Aggy" },
    { name: "description", content: "Aggy is an experiment." },
  ];
};

export const loader = async ({ request, params }: LoaderArgs) => {
  const { response, client } = newApiClient({ request });


  const topPosts = await client.getTopPosts();
  return response.ok({
    topPosts
  });
}

export default function Index() {
  // const [topPosts] = getTopPosts.use();
  const { topPosts } = useLoaderData<typeof loader>();

  return (
    <RootLayout>
      <ol className="flex flex-col gap-2">
        {
          topPosts.map(post =>
            <li className="" key={post.id}>
              <a className="text-xl" href={post.link}>{post.title}</a>
              <PostStatusLines post={post} />
            </li>
          )
        }
      </ol>
    </RootLayout>
  );
}

