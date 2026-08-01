using Microsoft.EntityFrameworkCore.Storage.ValueConversion;

namespace BerryAIGen.Infrastructure.Data.Converters;

/// <summary>
/// Generic value converter for nullable strongly-typed IDs (record types).
/// </summary>
/// <typeparam name="TId">The strongly-typed ID type.</typeparam>
public class NullableStronglyTypedIdConverter<TId> : ValueConverter<TId?, Guid?>
    where TId : class
{
    private static readonly Type IdType = typeof(TId);
    private static readonly Func<Guid, TId> FromGuid = CreateFromGuidFactory();

    /// <summary>
    /// Initializes a new instance of the NullableStronglyTypedIdConverter class.
    /// </summary>
    public NullableStronglyTypedIdConverter()
        : base(
            id => id != null ? GetGuidValue(id) : null,
            guid => guid.HasValue ? FromGuid(guid.Value) : null)
    {
    }

    private static Guid GetGuidValue(TId id)
    {
        var property = IdType.GetProperty("Value");
        if (property == null)
            throw new InvalidOperationException($"Type {IdType.Name} does not have a 'Value' property.");

        return (Guid)property.GetValue(id)!;
    }

    private static Func<Guid, TId> CreateFromGuidFactory()
    {
        var fromMethod = IdType.GetMethod("From", new[] { typeof(Guid) });
        if (fromMethod == null)
            throw new InvalidOperationException($"Type {IdType.Name} does not have a 'From(Guid)' method.");

        return guid => (TId)fromMethod.Invoke(null, new object[] { guid })!;
    }
}