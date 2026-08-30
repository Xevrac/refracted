using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_Ungarrisoned
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.Ungarrisoned); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.Ungarrisoned)obj;
            //  Serialize GarrisonPlayerId
            s.Write(value.GarrisonPlayerId);
            //  Serialize GarrisonId
            s.Write(value.GarrisonId);
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize UnitId
            s.Write(value.UnitId);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Facing
            s.Write(value.Facing);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.Ungarrisoned)) as Rts.CnC.Messages.Client.Ungarrisoned;
            //  Deserialize GarrisonPlayerId
            s.Read(out value.GarrisonPlayerId);
            //  Deserialize GarrisonId
            s.Read(out value.GarrisonId);
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize UnitId
            s.Read(out value.UnitId);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Facing
            s.Read(out value.Facing);

            return value;
        }
        
    }
}
