
import { topPosts } from "@/client"
import { PostStatusLines } from "@/app/_components";


export default function Home() {
  return (
    <>
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
    </>
  );
}

