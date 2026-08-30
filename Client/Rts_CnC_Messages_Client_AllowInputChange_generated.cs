using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AllowInputChange
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AllowInputChange); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AllowInputChange)obj;
            //  Serialize Enable
            s.Write(value.Enable);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AllowInputChange)) as Rts.CnC.Messages.Client.AllowInputChange;
            //  Deserialize Enable
            s.Read(out value.Enable);

            return value;
        }
        
    }
}
