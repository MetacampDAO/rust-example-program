import * as web3 from '@solana/web3.js'
import * as borsh from "borsh"

import {initializeSolSignerKeypair, airdropSolIfNeeded } from './initializeKeypair'

main().then(() => {
    console.log('Finished successfully')
    process.exit(0)
}).catch(error => {
    console.log(error)
    process.exit(1)
})

async function main() {
      
    const connection = new web3.Connection(web3.clusterApiUrl('devnet'))
    const signer = initializeSolSignerKeypair()  
    await airdropSolIfNeeded(connection, signer.publicKey, 2, 0.05)
    
    const onChainProgramId = new web3.PublicKey('4bNwLdGiPRdVGvzojLFsGc3fz9fxSzjcpCUPHzrjbFUT')
    await hello(signer, onChainProgramId, connection)
    // await initializeOnchainAccount(signer, onChainProgramId, connection)

}

async function hello(signer: web3.Keypair, programId: web3.PublicKey, connection: web3.Connection) {
    // Create new transaction
    const transaction = new web3.Transaction()

    // Set instruction data
    const instructionId = 0
    const instructionData = new onchainInstructionData(
        {
            instruction: instructionId
        }
    )

    // Encode instruction data according to buffer layout
    let serializedInstructionData = Buffer.from(
        borsh.serialize(
            helloInstructionDataSchema,
            instructionData
        )
    )

    // Create instruction
    const instruction = new web3.TransactionInstruction({
        programId: programId,
        keys: [
            {
                pubkey: signer.publicKey,
                isSigner: true,
                isWritable: false
            }
        ],
        data: serializedInstructionData
    })

    // Add instruction to transaction
    transaction.add(instruction)

    // Sign transaction and send to via connection to RPC endpoint
    const tx = await web3.sendAndConfirmTransaction(connection, transaction, [signer])

    console.log(`https://explorer.solana.com/tx/${tx}?cluster=devnet`)
}




async function initializeOnchainAccount(signer: web3.Keypair, programId: web3.PublicKey, connection: web3.Connection) {
    
    // Create new transaction
    const transaction = new web3.Transaction()

    // Set instruction data
    const instructionId = 1
    const accountId = 11
    const name = "test"
    let instructionData = new onchainInstructionData(
        {
            instruction: instructionId,
            id: accountId,
            name: name
        }
    )

    // Encode instruction data according to buffer layout
    let serializedInstructionData = Buffer.from(
        borsh.serialize(
            onchainInstructionDataSchema,
            instructionData
        )
    )

    console.log(`Buffer length: ${serializedInstructionData.length}`)    

    // Derive PDA with seeds of accountId and signer's publickey
    const [pda] = await web3.PublicKey.findProgramAddressSync(
        [signer.publicKey.toBuffer(), Uint8Array.of(accountId)],
        programId
    )
    console.log("PDA is:", pda.toBase58())

    // Create instruction
    const instruction = new web3.TransactionInstruction({
        programId: programId,
        data: serializedInstructionData,
        keys: [
            {
                pubkey: signer.publicKey,
                isSigner: true,
                isWritable: false
            },
            {
                pubkey: pda,
                isSigner: false,
                isWritable: true
            },
            {
                pubkey: web3.SystemProgram.programId,
                isSigner: false,
                isWritable: false
            }
        ]
    })

    // Add instruction to transaction
    transaction.add(instruction)

    // Sign transaction and send to via connection to RPC endpoint
    const tx = await web3.sendAndConfirmTransaction(connection, transaction, [signer])

    console.log(`https://explorer.solana.com/tx/${tx}?cluster=devnet`)
}




// Define instruction buffer
class onchainInstructionData {
    instruction;
    id?;
    name?;
    constructor(fields: { instruction: number, id?: number, name?: string } | undefined = undefined) {
        if (fields) {
        this.instruction = fields.instruction;
        this.id = fields.id;
        this.name = fields.name;
        }
    }
}
  
/**
 * Borsh schema definition for greeting state account
 */
const helloInstructionDataSchema = new Map([
[
    onchainInstructionData, 
    {   
        kind: "struct", 
        fields: [
            ["instruction", "u8"]
        ] 
    }
],
]);

const onchainInstructionDataSchema = new Map([
    [
        onchainInstructionData, 
        {   
            kind: "struct", 
            fields: [
                ["instruction", "u8"],
                ["id", "u8"],
                ["name", "string"]
            ] 
        }
    ],
    ]);


