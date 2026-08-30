using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_StructurePowerChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.StructurePowerChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.StructurePowerChanged)obj;
            //  Serialize PowerProduced
            s.Write(value.PowerProduced);
            //  Serialize PowerUsed
            s.Write(value.PowerUsed);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.StructurePowerChanged)) as Rts.CnC.Messages.Client.StructurePowerChanged;
            //  Deserialize PowerProduced
            s.Read(out value.PowerProduced);
            //  Deserialize PowerUsed
            s.Read(out value.PowerUsed);

            return value;
        }
        
    }
}
