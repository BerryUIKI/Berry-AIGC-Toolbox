namespace BerryAIGen.Domain.ValueObjects;

/// <summary>
/// Represents a unique identifier for a Folder entity.
/// </summary>
public sealed record FolderId
{
    public Guid Value { get; }

    private FolderId(Guid value)
    {
        Value = value;
    }

    public static FolderId New() => new(Guid.NewGuid());
    public static FolderId From(Guid value) => new(value);

    public override string ToString() => Value.ToString();

    public static implicit operator Guid(FolderId id) => id.Value;
    public static explicit operator FolderId(Guid value) => From(value);
}