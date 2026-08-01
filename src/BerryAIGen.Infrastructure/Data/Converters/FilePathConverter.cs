using Microsoft.EntityFrameworkCore.Storage.ValueConversion;
using BerryAIGen.Domain.ValueObjects;
using System.Reflection;

namespace BerryAIGen.Infrastructure.Data.Converters;

/// <summary>
/// Value converter for FilePath value object.
/// Uses reflection to bypass factory method validation in expression trees.
/// </summary>
public class FilePathConverter : ValueConverter<FilePath, string>
{
    private static readonly TypeInfo FilePathTypeInfo = typeof(FilePath).GetTypeInfo();
    private static readonly ConstructorInfo FilePathConstructor = FilePathTypeInfo
        .DeclaredConstructors
        .First(c => c.GetParameters().Length == 1 && c.GetParameters()[0].ParameterType == typeof(string));

    /// <summary>
    /// Initializes a new instance of the FilePathConverter class.
    /// </summary>
    public FilePathConverter()
        : base(
            filePath => filePath.Value,
            value => CreateFilePath(value))
    {
    }

    private static FilePath CreateFilePath(string value)
    {
        // Use private constructor to bypass validation
        // Validation already happened when entity was created
        return (FilePath)FilePathConstructor.Invoke(new object[] { value });
    }
}