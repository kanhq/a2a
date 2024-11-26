import a2a

print(a2a)

print(dir(a2a)) 

async def main():
  body = await a2a.do_action({
    "kind": "file",
    "method": "READ",
    "path": "../../Cargo.toml"
  })
  print(body)


if __name__ == "__main__":
  import asyncio
  asyncio.run(main())


