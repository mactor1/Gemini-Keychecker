use crate::error::ValidatorError;
use crate::types::ValidatedKey;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub async fn write_key_into_file<W>(
    writer: &mut W,
    validated_key: &ValidatedKey,
) -> Result<(), ValidatorError>
where
    W: AsyncWrite + Unpin,
{
    writer
        .write_all(format!("{}\n", validated_key.key.as_ref()).as_bytes())
        .await?;
    Ok(())
}
