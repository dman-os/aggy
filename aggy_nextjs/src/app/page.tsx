
import { PostStatusLines } from "@/app/_components";
import { getCsrfToken } from "@/utils/index.server";
import { apiClient } from '@/client/index.server';


export default async function Home() {
  const { client, session } = apiClient();
  const topPosts = await client.getTopPosts();
  return (
    <>
      <ol className="flex flex-col gap-2">
        {
          topPosts.items.map(post =>
            <li className="" key={post.id}>
              <a className="text-xl" href={post.url ?? `/p/${post.id}`}>{post.title}</a>
              <PostStatusLines post={post} csrfToken={getCsrfToken()} />
            </li>
          )
        }
      </ol>
    </>
  );
}

