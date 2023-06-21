import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { InoooStaking } from "../target/types/inooo_staking";

describe("inooo-staking", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.InoooStaking as Program<InoooStaking>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
