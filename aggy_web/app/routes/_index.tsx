import { LoaderArgs, type V2_MetaFunction } from "@remix-run/node";
import { Form, useLoaderData } from "@remix-run/react";
import { getTopPosts } from "~/api";
import { newApiClient } from "~/api/index.server";
import { ThemeToggle } from "~/components/ThemeToggle";

export const meta: V2_MetaFunction = () => {
  return [
    { title: "Aggy" },
    { name: "description", content: "News aggregrator." },
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
    <>
      <header className="flex gap-2 px-2rem py-2rem">
        <div>Aggy</div>
        <nav className="mx-a flex gap-2">
          <a href="new">new</a>
          <a href="comments">comments</a>
          <a href="submit">submit</a>
        </nav>
        <ThemeToggle className="p-2 navbar flex min-h-5rem" />
      </header>
      <main>
        <ol className="flex flex-col gap-2">
          {
            topPosts.map(post =>
              <li className="" key={post.id}>
                <a className="text-xl" href={post.link}>{post.title}</a>
                <div className="flex gap-1">
                  <span>
                    by <a href={`/user/${post.epigram.author.pkey}`}>{post.epigram.author.alias}</a>
                  </span>
                  |
                  <a href={`/post/${post.id}`}>{post.commentCount} comments</a>
                </div>
                <div className="flex gap-1">
                  {Object.entries(post.epigram.topFaces).map(([rxn, { count, userFacedAtTs }]) =>
                    <Form
                      key={post.id}
                      className="inline-block"
                      method="post"
                      action={
                        userFacedAtTs ?
                          `/unface?epigram_id=${post.epigram.id}&rxn=${rxn}`
                          : `/doface?epigram_id=${post.epigram.id}&rxn=${rxn}`
                      }
                    >
                      <button
                        type="submit"
                        className="p-1 b-1 rounded-2 b-outline hover:b-black dark:hover:b-white data-[faced]:b-orange"
                        {...(userFacedAtTs ? { 'data-faced': !!userFacedAtTs } : {})}
                      >
                        <span className="">{rxn} </span>
                        <span className="italic">{count}</span>
                      </button>
                    </Form>
                  )}
                </div>
              </li>
            )
          }
        </ol>
      </main >
    </>
  );
}
